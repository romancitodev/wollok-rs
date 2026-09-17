//! AST (`wollok-ast`) -> bytecode (`wollok-vm`) compiler.
//!
//! Scope, matched to what the VM can actually execute today (see
//! `docs/vm-design.md`): classes and singleton `object`s with fields and
//! methods (field initializers run as a synthesized constructor — see
//! `compile_ctor`), `new`, field/global access and assignment, locals,
//! `if`, `return`, message sends (including operators, compiled as sends),
//! string/number/boolean literals, and top-level statements run as an
//! implicit `main` method.
//!
//! Not supported yet, because the VM has no instructions for them:
//! arrays/sets, closures, try/catch, `super`, and `new`-call constructor
//! arguments (still discarded — only the zero-arg field initializers run).
//! See `docs/backlog.md` for what's next and why.

use hashbrown::HashMap;

use wollok_ast::{
    ast::{Scope, Stmt},
    expr::{Block, Expr},
    item::{Item, ItemClass, ItemObject},
};
use wollok_common::ast::{BinaryOp, UnaryOp};
use wollok_lexer::token::Literal;
use wollok_vm::{
    bytecode::{ConstIdx, GlobalIdx, Instr, InstrIdx, SlotIdx},
    dispatch::MethodRef,
    heap::{ClassId, FieldIdx},
    method::Method,
    program::Program,
    strings::StringTable,
    value::Value,
    vm::Vm,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// An AST shape the VM has no bytecode/value representation for yet.
    Unsupported(&'static str),
    UndefinedName(String),
    DuplicateClass(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Unsupported(what) => write!(f, "not supported yet: {what}"),
            CompileError::UndefinedName(name) => write!(f, "undefined name: {name}"),
            CompileError::DuplicateClass(name) => write!(f, "duplicate class: {name}"),
        }
    }
}

impl std::error::Error for CompileError {}

pub struct Compiled {
    pub program: Program,
    /// Every top-level statement (outside a class), run in order as a
    /// zero-arg method; its result is the last expression's value.
    pub main: MethodRef,
}

struct ClassInfo {
    id: ClassId,
    fields: Vec<String>,
}

/// A `class` and a singleton `object` compile the same way (fields +
/// methods, no superclass/mixin linearization yet) — this just picks
/// which `Item` field to read.
enum ClassLike<'a> {
    Class(&'a ItemClass),
    Object(&'a ItemObject),
}

impl<'a> ClassLike<'a> {
    fn name(&self) -> &'a str {
        match self {
            ClassLike::Class(c) => &c.name,
            ClassLike::Object(o) => &o.name,
        }
    }

    fn body(&self) -> &'a [Item] {
        match self {
            ClassLike::Class(c) => &c.body,
            ClassLike::Object(o) => &o.body,
        }
    }

    fn is_object(&self) -> bool {
        matches!(self, ClassLike::Object(_))
    }
}

/// Compiles a parsed `Scope` into a runnable `Program`. Reserves inline
/// cache slots and global slots on `vm` for every send/singleton `object`
/// compiled, so `vm` must be the same `Vm` the resulting program later
/// runs on.
pub fn compile(scope: &Scope, vm: &mut Vm) -> Result<Compiled, CompileError> {
    let class_like_items: Vec<ClassLike> = scope
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Item(Item::Class(class)) => Some(ClassLike::Class(class)),
            Stmt::Item(Item::Object(object)) => Some(ClassLike::Object(object)),
            _ => None,
        })
        .collect();

    let (classes, globals) = collect_classes(&class_like_items, vm)?;
    let mut program = Program::default();

    for item in &class_like_items {
        let info = &classes[item.name()];
        let mut vtable = HashMap::new();

        for member in item.body() {
            let method_item = match member {
                Item::Method(m) => m,
                Item::PrefixedMethod(prefixed) => &prefixed.method,
                Item::Const(_) | Item::Let(_) | Item::Property(_) => continue,
                _ => return Err(CompileError::Unsupported("item inside a class/object body")),
            };
            // Abstract methods have no body; nothing to run (yet).
            let Some(body) = &method_item.body else {
                continue;
            };
            let arity = u8::try_from(method_item.signature.params.len())
                .map_err(|_| CompileError::Unsupported("more than 255 params"))?;

            let mut mc = MethodCompiler {
                selectors: &mut program.selectors,
                consts: &mut program.consts,
                strings: &mut vm.strings,
                caches: &mut vm.caches,
                classes: &classes,
                globals: &globals,
                current_class: Some(info),
                locals: HashMap::new(),
                next_slot: u32::from(arity),
                code: Vec::new(),
            };
            for (i, param) in method_item.signature.params.iter().enumerate() {
                mc.locals
                    .insert(param.name.clone(), SlotIdx(u32::try_from(i).unwrap()));
            }
            mc.compile_block(&body.stmts)?;
            mc.emit(Instr::Return);
            let extra_locals = mc.next_slot - u32::from(arity);
            let code = mc.code;

            let selector = program
                .selectors
                .intern(&method_item.signature.ident, arity);
            let method_ref = program.methods.define(Method {
                selector,
                arity,
                extra_locals,
                code,
            });
            vtable.insert(selector, method_ref);
        }

        let ctor = compile_ctor(item, info, &classes, &globals, &mut program, vm)?;

        let class_id = program.classes.define(
            item.name().to_owned(),
            info.fields.len() as u32,
            ctor,
            vtable,
        );
        debug_assert_eq!(
            class_id, info.id,
            "classes/objects must be compiled in the same order collect_classes assigned ids in"
        );
    }

    let top_level: Vec<Stmt> = scope
        .iter()
        .filter(|stmt| !matches!(stmt, Stmt::Item(Item::Class(_) | Item::Object(_))))
        .cloned()
        .collect();

    let mut mc = MethodCompiler {
        selectors: &mut program.selectors,
        consts: &mut program.consts,
        strings: &mut vm.strings,
        caches: &mut vm.caches,
        classes: &classes,
        globals: &globals,
        current_class: None,
        locals: HashMap::new(),
        next_slot: 0,
        code: Vec::new(),
    };

    // Singletons are instantiated once, before any user code runs, and
    // bound to their reserved global slot — in file order.
    for item in &class_like_items {
        if !item.is_object() {
            continue;
        }
        let info = &classes[item.name()];
        let slot = globals[item.name()];
        mc.emit(Instr::NewInstance {
            class: info.id,
            arg_count: 0,
        });
        mc.emit(Instr::StoreGlobal(slot));
    }

    mc.compile_block(&top_level)?;
    mc.emit(Instr::Return);
    let extra_locals = mc.next_slot;
    let code = mc.code;

    let main_selector = program.selectors.intern("main", 0);
    let main = program.methods.define(Method {
        selector: main_selector,
        arity: 0,
        extra_locals,
        code,
    });

    Ok(Compiled { program, main })
}

/// Compiles a class/object's field initializers into a zero-arg
/// constructor method, run by `NewInstance` on the freshly allocated
/// instance (`self` = the new object) before it's returned. `None` when
/// there are no fields to initialize.
fn compile_ctor(
    item: &ClassLike,
    info: &ClassInfo,
    classes: &HashMap<String, ClassInfo>,
    globals: &HashMap<String, GlobalIdx>,
    program: &mut Program,
    vm: &mut Vm,
) -> Result<Option<MethodRef>, CompileError> {
    if info.fields.is_empty() {
        return Ok(None);
    }

    let mut mc = MethodCompiler {
        selectors: &mut program.selectors,
        consts: &mut program.consts,
        strings: &mut vm.strings,
        caches: &mut vm.caches,
        classes,
        globals,
        current_class: Some(info),
        locals: HashMap::new(),
        next_slot: 0,
        code: Vec::new(),
    };

    let mut field_idx = 0u32;
    for member in item.body() {
        let init_expr = match member {
            Item::Const(c) => &c.expr,
            Item::Let(l) => &l.expr,
            Item::Property(p) => &p.expr,
            _ => continue,
        };
        mc.compile_expr(init_expr)?;
        mc.emit(Instr::StoreField(FieldIdx(field_idx)));
        field_idx += 1;
    }
    mc.emit(Instr::PushNull);
    mc.emit(Instr::Return);
    let extra_locals = mc.next_slot;
    let code = mc.code;

    // Bracketed, arity-0 synthetic name: no user selector can ever collide
    // with it (Wollok identifiers don't contain `<`/`:`/`>`).
    let selector = program
        .selectors
        .intern(&format!("<init:{}>", item.name()), 0);
    Ok(Some(program.methods.define(Method {
        selector,
        arity: 0,
        extra_locals,
        code,
    })))
}

/// First pass: assigns every class/object a `ClassId` (matching the order
/// `Program::classes.define` will later be called in — see the
/// `debug_assert_eq!` above), records its field names in declaration
/// order, and reserves a global slot for every singleton `object` — all
/// before any method body gets compiled, so bodies can forward-reference
/// classes/objects declared later in the file.
type ClassesAndGlobals = (HashMap<String, ClassInfo>, HashMap<String, GlobalIdx>);

fn collect_classes(items: &[ClassLike], vm: &mut Vm) -> Result<ClassesAndGlobals, CompileError> {
    let mut classes = HashMap::new();
    let mut globals = HashMap::new();

    for (i, item) in items.iter().enumerate() {
        let name = item.name();
        if classes.contains_key(name) {
            return Err(CompileError::DuplicateClass(name.to_string()));
        }

        let fields = item
            .body()
            .iter()
            .filter_map(|member| match member {
                Item::Const(c) => Some(c.name.clone()),
                Item::Let(l) => Some(l.name.clone()),
                Item::Property(p) => Some(p.name.clone()),
                _ => None,
            })
            .collect();

        classes.insert(
            name.to_owned(),
            ClassInfo {
                id: ClassId(u32::try_from(i).unwrap()),
                fields,
            },
        );
        if item.is_object() {
            globals.insert(name.to_owned(), vm.reserve_global());
        }
    }

    Ok((classes, globals))
}

/// Compiles one method (or the synthetic top-level `main`) body into flat
/// bytecode. Locals are a single flat namespace per method (no per-block
/// shadowing) — matches what a stack of slots buys you without a real
/// scope stack, and nothing in this compiler's scope needs shadowing yet.
struct MethodCompiler<'a> {
    selectors: &'a mut wollok_vm::selector::SelectorTable,
    consts: &'a mut Vec<Value>,
    strings: &'a mut StringTable,
    caches: &'a mut wollok_vm::dispatch::InlineCacheTable,
    classes: &'a HashMap<String, ClassInfo>,
    /// Singleton `object`s, reachable by name from anywhere, not just the
    /// enclosing class/method.
    globals: &'a HashMap<String, GlobalIdx>,
    current_class: Option<&'a ClassInfo>,
    locals: HashMap<String, SlotIdx>,
    next_slot: u32,
    code: Vec<Instr>,
}

impl MethodCompiler<'_> {
    fn emit(&mut self, instr: Instr) -> usize {
        self.code.push(instr);
        self.code.len() - 1
    }

    /// Patches a previously emitted `Jump`/`JumpIfFalse` at `idx` to land
    /// on the next instruction that gets emitted.
    fn patch_to_here(&mut self, idx: usize) {
        let target = InstrIdx(u32::try_from(self.code.len()).unwrap());
        match &mut self.code[idx] {
            Instr::Jump(t) | Instr::JumpIfFalse(t) => *t = target,
            other => unreachable!("patch_to_here on a non-jump instruction: {other:?}"),
        }
    }

    fn declare_local(&mut self, name: String) -> SlotIdx {
        let slot = SlotIdx(self.next_slot);
        self.next_slot += 1;
        self.locals.insert(name, slot);
        slot
    }

    fn field_index(&self, name: &str) -> Option<u32> {
        let idx = self.current_class?.fields.iter().position(|f| f == name)?;
        Some(u32::try_from(idx).unwrap())
    }

    fn emit_send(&mut self, name: &str, arg_count: u8) {
        let method_name = self.selectors.intern(name, arg_count);
        let cache_slot = self.caches.reserve_slot();
        self.emit(Instr::Send {
            method_name,
            arg_count,
            cache_slot,
        });
    }

    /// Compiles a sequence of statements so it leaves exactly one value on
    /// the stack: the last expression statement's value, or `Null` if the
    /// sequence is empty or ends in a declaration.
    fn compile_block(&mut self, stmts: &[Stmt]) -> Result<(), CompileError> {
        let Some(last) = stmts.len().checked_sub(1) else {
            self.emit(Instr::PushNull);
            return Ok(());
        };

        for (i, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Expr(expr) => {
                    self.compile_expr(expr)?;
                    if i != last {
                        self.emit(Instr::Pop);
                    }
                }
                Stmt::Item(Item::Let(l)) => {
                    self.compile_expr(&l.expr)?;
                    let slot = self.declare_local(l.name.clone());
                    self.emit(Instr::StoreLocal(slot));
                    if i == last {
                        self.emit(Instr::PushNull);
                    }
                }
                Stmt::Item(Item::Const(c)) => {
                    self.compile_expr(&c.expr)?;
                    let slot = self.declare_local(c.name.clone());
                    self.emit(Instr::StoreLocal(slot));
                    if i == last {
                        self.emit(Instr::PushNull);
                    }
                }
                _ => return Err(CompileError::Unsupported("statement inside a method body")),
            }
        }
        Ok(())
    }

    /// Every case here must leave exactly one value on the stack.
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Lit(lit) => self.compile_lit(&lit.value)?,
            Expr::Self_ => {
                self.emit(Instr::PushSelf);
            }
            Expr::Paren(paren) => self.compile_expr(&paren.expr)?,
            Expr::Field(field) => self.compile_field_read(&field.base, &field.name)?,
            Expr::Assign(assign) => self.compile_assign(&assign.left, &assign.right)?,
            Expr::Call(call) => self.compile_call(&call.callee, &call.args)?,
            Expr::Class(new_expr) => self.compile_new(&new_expr.name, &new_expr.params)?,
            Expr::If(if_expr) => self.compile_if(
                &if_expr.condition,
                &if_expr.then,
                if_expr.otherwise.as_ref(),
            )?,
            Expr::Return(ret) => self.compile_return(ret.value.as_deref())?,
            Expr::Binary(bin) => self.compile_binary(&bin.left, &bin.right, &bin.op)?,
            Expr::Unary(un) => self.compile_unary(&un.op, &un.expr)?,
            Expr::Super_ => return Err(CompileError::Unsupported("super (no SendSuper yet)")),
            _ => return Err(CompileError::Unsupported("this expression kind")),
        }
        Ok(())
    }

    fn compile_lit(&mut self, lit: &Literal) -> Result<(), CompileError> {
        match lit {
            Literal::Integer(n) => {
                let idx = ConstIdx(u32::try_from(self.consts.len()).unwrap());
                self.consts.push(Value::from(*n));
                self.emit(Instr::PushConst(idx));
            }
            Literal::Float(n) => {
                let idx = ConstIdx(u32::try_from(self.consts.len()).unwrap());
                self.consts.push(Value::from(*n));
                self.emit(Instr::PushConst(idx));
            }
            Literal::Boolean(b) => {
                self.emit(if *b {
                    Instr::PushTrue
                } else {
                    Instr::PushFalse
                });
            }
            Literal::Null => {
                self.emit(Instr::PushNull);
            }
            Literal::String(s) => {
                let str_idx = self.strings.intern(s);
                let const_idx = ConstIdx(u32::try_from(self.consts.len()).unwrap());
                self.consts.push(Value::from(str_idx));
                self.emit(Instr::PushConst(const_idx));
            }
        }
        Ok(())
    }

    /// A bare `name` (self field/local) or `base.name` (getter send).
    fn compile_field_read(&mut self, base: &Expr, name: &str) -> Result<(), CompileError> {
        if matches!(base, Expr::Self_) {
            self.compile_name_ref(name)
        } else {
            self.compile_expr(base)?;
            self.emit_send(name, 0);
            Ok(())
        }
    }

    /// Resolves a bare name: nearest local first, then a field of the
    /// enclosing class, then a singleton `object` of that name. No
    /// fallback to a zero-arg self-send — Wollok lets a bare name mean
    /// "call this getter method", but this compiler can't tell that apart
    /// from a typo, so it errors instead of guessing.
    fn compile_name_ref(&mut self, name: &str) -> Result<(), CompileError> {
        if let Some(&slot) = self.locals.get(name) {
            self.emit(Instr::LoadLocal(slot));
        } else if let Some(idx) = self.field_index(name) {
            self.emit(Instr::LoadField(FieldIdx(idx)));
        } else if let Some(&slot) = self.globals.get(name) {
            self.emit(Instr::LoadGlobal(slot));
        } else {
            return Err(CompileError::UndefinedName(name.to_string()));
        }
        Ok(())
    }

    fn compile_assign(&mut self, left: &Expr, right: &Expr) -> Result<(), CompileError> {
        let Expr::Field(field) = left else {
            return Err(CompileError::Unsupported("assignment target"));
        };

        if matches!(*field.base, Expr::Self_) {
            self.compile_expr(right)?;
            if let Some(&slot) = self.locals.get(&field.name) {
                self.emit(Instr::StoreLocal(slot));
                self.emit(Instr::LoadLocal(slot));
            } else if let Some(idx) = self.field_index(&field.name) {
                self.emit(Instr::StoreField(FieldIdx(idx)));
                self.emit(Instr::LoadField(FieldIdx(idx)));
            } else {
                return Err(CompileError::UndefinedName(field.name.clone()));
            }
        } else {
            // `base.name = value` desugars to a setter send `base.name=(value)`.
            self.compile_expr(&field.base)?;
            self.compile_expr(right)?;
            self.emit_send(&format!("{}=", field.name), 1);
        }
        Ok(())
    }

    fn compile_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<(), CompileError> {
        let Expr::Field(field) = callee else {
            return Err(CompileError::Unsupported("call target"));
        };

        if matches!(*field.base, Expr::Self_) {
            self.emit(Instr::PushSelf);
        } else {
            self.compile_expr(&field.base)?;
        }
        for arg in args {
            self.compile_expr(arg)?;
        }
        let arg_count = u8::try_from(args.len())
            .map_err(|_| CompileError::Unsupported("more than 255 args"))?;
        self.emit_send(&field.name, arg_count);
        Ok(())
    }

    fn compile_new(&mut self, name: &str, params: &[Expr]) -> Result<(), CompileError> {
        let class_id = self
            .classes
            .get(name)
            .ok_or_else(|| CompileError::UndefinedName(name.to_string()))?
            .id;
        for param in params {
            self.compile_expr(param)?;
        }
        let arg_count = u8::try_from(params.len())
            .map_err(|_| CompileError::Unsupported("more than 255 constructor args"))?;
        self.emit(Instr::NewInstance {
            class: class_id,
            arg_count,
        });
        Ok(())
    }

    fn compile_if(
        &mut self,
        condition: &Expr,
        then: &Block,
        otherwise: Option<&Block>,
    ) -> Result<(), CompileError> {
        self.compile_expr(condition)?;
        let jump_if_false = self.emit(Instr::JumpIfFalse(InstrIdx(0)));
        self.compile_block(&then.stmts)?;
        let jump_to_end = self.emit(Instr::Jump(InstrIdx(0)));

        self.patch_to_here(jump_if_false);
        match otherwise {
            Some(block) => self.compile_block(&block.stmts)?,
            None => {
                self.emit(Instr::PushNull);
            }
        }
        self.patch_to_here(jump_to_end);
        Ok(())
    }

    fn compile_return(&mut self, value: Option<&Expr>) -> Result<(), CompileError> {
        match value {
            Some(expr) => self.compile_expr(expr)?,
            None => {
                self.emit(Instr::PushNull);
            }
        }
        self.emit(Instr::Return);
        Ok(())
    }

    fn compile_binary(
        &mut self,
        left: &Expr,
        right: &Expr,
        op: &BinaryOp,
    ) -> Result<(), CompileError> {
        match op {
            BinaryOp::And => self.compile_and(left, right),
            BinaryOp::Or => self.compile_or(left, right),
            _ => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.emit_send(binary_op_selector(op), 1);
                Ok(())
            }
        }
    }

    /// Short-circuiting `&&`: both operands must be booleans (matches the
    /// VM's `JumpIfFalse`, which panics on a non-boolean).
    fn compile_and(&mut self, left: &Expr, right: &Expr) -> Result<(), CompileError> {
        self.compile_expr(left)?;
        let jump_if_false = self.emit(Instr::JumpIfFalse(InstrIdx(0)));
        self.compile_expr(right)?;
        let jump_to_end = self.emit(Instr::Jump(InstrIdx(0)));
        self.patch_to_here(jump_if_false);
        self.emit(Instr::PushFalse);
        self.patch_to_here(jump_to_end);
        Ok(())
    }

    fn compile_or(&mut self, left: &Expr, right: &Expr) -> Result<(), CompileError> {
        self.compile_expr(left)?;
        let jump_if_false = self.emit(Instr::JumpIfFalse(InstrIdx(0)));
        self.emit(Instr::PushTrue);
        let jump_to_end = self.emit(Instr::Jump(InstrIdx(0)));
        self.patch_to_here(jump_if_false);
        self.compile_expr(right)?;
        self.patch_to_here(jump_to_end);
        Ok(())
    }

    fn compile_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Result<(), CompileError> {
        self.compile_expr(expr)?;
        match op {
            UnaryOp::Not => self.emit_send("negate", 0),
        }
        Ok(())
    }
}

fn binary_op_selector(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::Plus => "+",
        BinaryOp::Minus => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Div => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Pow => "**",
        BinaryOp::And | BinaryOp::Or => unreachable!("short-circuited before reaching here"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wollok_ast::ast::Scope;
    use wollok_lexer::lexer::TokenStream;

    fn compile_source(src: &str) -> (Compiled, Vm) {
        let scope = Scope::from_tokens("test", TokenStream::new(src));
        let mut vm = Vm::new();
        let compiled = compile(&scope, &mut vm).expect("compilation failed");
        (compiled, vm)
    }

    #[test]
    fn top_level_arithmetic_send_and_literal_flow() {
        // No native Int dispatch yet, so this only proves the "then" branch
        // (a literal) round-trips through compile -> run.
        let (compiled, mut vm) = compile_source("if (true) 111 else 222");
        let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
        assert_eq!(result.as_int(), Some(111));
    }

    #[test]
    fn class_with_field_get_set_via_self_sends() {
        let (compiled, mut vm) = compile_source(
            r"
            class Counter {
                let value = 0
                method setValue(v) { value = v }
                method value() = value
            }

            let c = new Counter()
            c.setValue(42)
            c.value()
            ",
        );
        let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn new_instance_runs_field_initializers() {
        let (compiled, mut vm) = compile_source(
            r"
            class Point {
                let x = 3
                method x() = x
            }

            let p = new Point()
            p.x()
            ",
        );
        let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
        assert_eq!(
            result.as_int(),
            Some(3),
            "field initializer must have run on `new`, not left the field Null"
        );
    }

    #[test]
    fn singleton_object_field_initializer_runs_before_first_use() {
        let (compiled, mut vm) = compile_source(
            r#"
            object pepita {
              const name = "pepita"

              method saludar() {
                return name
              }
            }

            pepita.saludar()
            "#,
        );
        let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
        let str_idx = result.as_str_idx().expect("expected a Str value");
        assert_eq!(vm.strings.get(str_idx), "pepita");
    }

    #[test]
    fn undefined_name_is_a_compile_error_not_a_panic() {
        let scope = Scope::from_tokens("test", TokenStream::new("nope"));
        let mut vm = Vm::new();
        let Err(err) = compile(&scope, &mut vm) else {
            panic!("expected a compile error");
        };
        assert_eq!(err, CompileError::UndefinedName("nope".to_string()));
    }

    #[test]
    fn singleton_object_method_call_returns_a_string_literal() {
        let (compiled, mut vm) = compile_source(
            r#"
            object pepita {
              property name = "pepita"

              method saludar() {
                return "hola, soy pepita!"
              }
            }

            pepita.saludar()
            "#,
        );
        let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
        let str_idx = result.as_str_idx().expect("expected a Str value");
        assert_eq!(vm.strings.get(str_idx), "hola, soy pepita!");
    }
}
