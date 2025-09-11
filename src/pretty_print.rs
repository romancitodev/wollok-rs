use owo_colors::OwoColorize;
use std::fmt::Write as _;
use wollok_ast::{
    ast::{Scope, Stmt},
    expr::{
        Block, Expr, ExprArray, ExprAssign, ExprBinary, ExprCall, ExprClass, ExprClosure,
        ExprConst, ExprField, ExprIf, ExprLet, ExprLit, ExprMethodCall, ExprNew, ExprObject,
        ExprParen, ExprReturn, ExprSet, ExprSuper, ExprTry, ExprTryBlock, ExprTuple, ExprUnary,
    },
    item::{
        Item, ItemClass, ItemConst, ItemImport, ItemLet, ItemMethod, ItemObject, ItemPackage,
        ItemProgram, ItemProperty, ItemTest,
    },
};
use wollok_common::ast::{BinaryOp, UnaryOp};
use wollok_lexer::token::Literal; // import without risk of name clashing

/// Configuration for pretty printing
#[derive(Debug, Clone)]
pub struct PrettyConfig {
    pub use_colors: bool,
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self { use_colors: true }
    }
}

/// A pretty printer for Wollok AST nodes with color support
pub struct PrettyPrinter {
    config: PrettyConfig,
    indent_level: usize,
    output: String,
}

impl PrettyPrinter {
    pub fn new(config: PrettyConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn print_scope(&mut self, scope: &Scope) -> String {
        self.output.clear();
        self.indent_level = 0;

        if self.config.use_colors {
            self.output
                .push_str(&format!("{}\n", "🌳 Wollok AST Scope".bright_blue().bold()));
        } else {
            self.output.push_str("🌳 Wollok AST Scope\n");
        }

        self.output.push_str(&"═".repeat(50));
        self.output.push('\n');

        for (i, stmt) in scope.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.print_stmt(stmt);
        }

        self.output.clone()
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn print_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Item(item) => self.print_item(item),
            Stmt::Expr(expr) => {
                self.output.push_str(&format!(
                    "{}📝 {}\n",
                    self.indent(),
                    if self.config.use_colors {
                        "Expression".cyan().to_string()
                    } else {
                        "Expression".to_string()
                    }
                ));
                self.indent_level += 1;
                self.print_expr(expr);
                self.indent_level -= 1;
            }
        }
    }

    fn print_item(&mut self, item: &Item) {
        match item {
            Item::Const(const_item) => self.print_const(const_item),
            Item::Let(let_item) => self.print_let(let_item),
            Item::Property(prop_item) => self.print_property(prop_item),
            Item::Method(method_item) => self.print_method(method_item),
            Item::Class(class_item) => self.print_class(class_item),
            Item::Object(object_item) => self.print_object(object_item),
            Item::Import(import_item) => self.print_import(import_item),
            Item::Test(test_item) => self.print_test(test_item),
            Item::Program(program_item) => self.print_program(program_item),
            Item::Package(package_item) => self.print_package(package_item),
            _ => {
                let ident = self.indent();
                let buffer = &mut self.output;
                _ = writeln!(
                    buffer,
                    "{}❓ {}",
                    ident,
                    if self.config.use_colors {
                        "Unknown Item".red().to_string()
                    } else {
                        "Unknown Item".to_string()
                    }
                );
            }
        }
    }

    fn print_const(&mut self, const_item: &ItemConst) {
        let name_colored = if self.config.use_colors {
            const_item.name.bright_yellow().to_string()
        } else {
            const_item.name.clone()
        };

        let const_colored = if self.config.use_colors {
            "const".bright_magenta().to_string()
        } else {
            "const".to_string()
        };

        let equals = if self.config.use_colors {
            " = ".white().to_string()
        } else {
            " = ".to_string()
        };

        self.output.push_str(&format!(
            "{}🔒 {} {}{}\n",
            self.indent(),
            const_colored,
            name_colored,
            equals
        ));
        self.indent_level += 1;
        self.print_expr(&const_item.expr);
        self.indent_level -= 1;
    }

    fn print_let(&mut self, let_item: &ItemLet) {
        let name_colored = if self.config.use_colors {
            let_item.name.bright_yellow().to_string()
        } else {
            let_item.name.clone()
        };

        let let_colored = if self.config.use_colors {
            "let".bright_cyan().to_string()
        } else {
            "let".to_string()
        };

        let equals = if self.config.use_colors {
            " = ".white().to_string()
        } else {
            " = ".to_string()
        };

        self.output.push_str(&format!(
            "{}🔓 {} {}{}\n",
            self.indent(),
            let_colored,
            name_colored,
            equals
        ));
        self.indent_level += 1;
        self.print_expr(&let_item.expr);
        self.indent_level -= 1;
    }

    fn print_property(&mut self, prop_item: &ItemProperty) {
        let name_colored = if self.config.use_colors {
            prop_item.name.bright_yellow().to_string()
        } else {
            prop_item.name.clone()
        };

        let property_colored = if self.config.use_colors {
            "property".bright_blue().to_string()
        } else {
            "property".to_string()
        };

        let equals = if self.config.use_colors {
            " = ".white().to_string()
        } else {
            " = ".to_string()
        };

        self.output.push_str(&format!(
            "{}🏷️  {} {}{}\n",
            self.indent(),
            property_colored,
            name_colored,
            equals
        ));
        self.indent_level += 1;
        self.print_expr(&prop_item.expr);
        self.indent_level -= 1;
    }

    fn print_method(&mut self, method_item: &ItemMethod) {
        let params_str = method_item
            .signature
            .params
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let method_colored = if self.config.use_colors {
            "method".bright_green().to_string()
        } else {
            "method".to_string()
        };

        let name_colored = if self.config.use_colors {
            method_item.signature.ident.bright_yellow().to_string()
        } else {
            method_item.signature.ident.clone()
        };

        let params_colored = if self.config.use_colors {
            params_str.cyan().to_string()
        } else {
            params_str
        };

        let parens = if self.config.use_colors {
            ("(".white().to_string(), ")".white().to_string())
        } else {
            ("(".to_string(), ")".to_string())
        };

        self.output.push_str(&format!(
            "{}⚡ {} {}{}{}{}\n",
            self.indent(),
            method_colored,
            name_colored,
            parens.0,
            params_colored,
            parens.1
        ));

        self.indent_level += 1;
        self.print_block(&method_item.body);
        self.indent_level -= 1;
    }

    fn print_class(&mut self, class_item: &ItemClass) {
        let superclass_str = if let Some(ref super_name) = class_item.superclass {
            if self.config.use_colors {
                format!(
                    " {} {}",
                    "extends".bright_magenta(),
                    super_name.bright_yellow()
                )
            } else {
                format!(" extends {}", super_name)
            }
        } else {
            String::new()
        };

        let class_colored = if self.config.use_colors {
            "class".bright_red().to_string()
        } else {
            "class".to_string()
        };

        let name_colored = if self.config.use_colors {
            class_item.name.bright_yellow().to_string()
        } else {
            class_item.name.clone()
        };

        let brace = if self.config.use_colors {
            "{".white().to_string()
        } else {
            "{".to_string()
        };

        self.output.push_str(&format!(
            "{}🏛️  {} {}{} {}\n",
            self.indent(),
            class_colored,
            name_colored,
            superclass_str,
            brace
        ));

        self.indent_level += 1;
        for item in &class_item.body {
            self.print_item(item);
        }
        self.indent_level -= 1;

        let closing_brace = if self.config.use_colors {
            "}".white().to_string()
        } else {
            "}".to_string()
        };

        self.output
            .push_str(&format!("{}{}\n", self.indent(), closing_brace));
    }

    fn print_object(&mut self, object_item: &ItemObject) {
        let object_colored = if self.config.use_colors {
            "object".bright_red().to_string()
        } else {
            "object".to_string()
        };

        let name_colored = if self.config.use_colors {
            object_item.name.bright_yellow().to_string()
        } else {
            object_item.name.clone()
        };

        let brace = if self.config.use_colors {
            "{".white().to_string()
        } else {
            "{".to_string()
        };

        self.output.push_str(&format!(
            "{}📦 {} {} {}\n",
            self.indent(),
            object_colored,
            name_colored,
            brace
        ));

        self.indent_level += 1;
        for item in &object_item.body {
            self.print_item(item);
        }
        self.indent_level -= 1;

        let closing_brace = if self.config.use_colors {
            "}".white().to_string()
        } else {
            "}".to_string()
        };

        self.output
            .push_str(&format!("{}{}\n", self.indent(), closing_brace));
    }

    fn print_import(&mut self, import_item: &ItemImport) {
        let import_str = if import_item.wildcard {
            format!("{}.*", import_item.module)
        } else {
            import_item.module.clone()
        };

        let import_colored = if self.config.use_colors {
            "import".bright_magenta().to_string()
        } else {
            "import".to_string()
        };

        let module_colored = if self.config.use_colors {
            import_str.cyan().to_string()
        } else {
            import_str
        };

        self.output.push_str(&format!(
            "{}📚 {} {}\n",
            self.indent(),
            import_colored,
            module_colored
        ));
    }

    fn print_test(&mut self, test_item: &ItemTest) {
        let test_colored = if self.config.use_colors {
            "test".bright_green().to_string()
        } else {
            "test".to_string()
        };

        let name_colored = if self.config.use_colors {
            test_item.name.bright_yellow().to_string()
        } else {
            test_item.name.clone()
        };

        self.output.push_str(&format!(
            "{}🧪 {} {}\n",
            self.indent(),
            test_colored,
            name_colored
        ));
        self.indent_level += 1;
        self.print_expr(&test_item.body);
        self.indent_level -= 1;
    }

    fn print_program(&mut self, program_item: &ItemProgram) {
        let program_colored = if self.config.use_colors {
            "program".bright_blue().to_string()
        } else {
            "program".to_string()
        };

        let name_colored = if self.config.use_colors {
            program_item.name.bright_yellow().to_string()
        } else {
            program_item.name.clone()
        };

        self.output.push_str(&format!(
            "{}🎯 {} {}\n",
            self.indent(),
            program_colored,
            name_colored
        ));
        self.indent_level += 1;
        self.print_expr(&program_item.body);
        self.indent_level -= 1;
    }

    fn print_package(&mut self, package_item: &ItemPackage) {
        let package_colored = if self.config.use_colors {
            "package".bright_magenta().to_string()
        } else {
            "package".to_string()
        };

        let name_colored = if self.config.use_colors {
            package_item.name.bright_yellow().to_string()
        } else {
            package_item.name.clone()
        };

        let brace = if self.config.use_colors {
            "{".white().to_string()
        } else {
            "{".to_string()
        };

        self.output.push_str(&format!(
            "{}📦 {} {} {}\n",
            self.indent(),
            package_colored,
            name_colored,
            brace
        ));

        self.indent_level += 1;
        for item in &package_item.body {
            self.print_item(item);
        }
        self.indent_level -= 1;

        let closing_brace = if self.config.use_colors {
            "}".white().to_string()
        } else {
            "}".to_string()
        };

        self.output
            .push_str(&format!("{}{}\n", self.indent(), closing_brace));
    }

    fn print_block(&mut self, block: &Block) {
        let brace_open = if self.config.use_colors {
            "{".white().to_string()
        } else {
            "{".to_string()
        };

        self.output
            .push_str(&format!("{}{}\n", self.indent(), brace_open));

        self.indent_level += 1;
        for expr in &block.stmts {
            self.print_expr(expr);
        }
        self.indent_level -= 1;

        let brace_close = if self.config.use_colors {
            "}".white().to_string()
        } else {
            "}".to_string()
        };

        self.output
            .push_str(&format!("{}{}\n", self.indent(), brace_close));
    }

    fn print_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Array(arr) => self.print_array(arr),
            Expr::Set(set) => self.print_set(set),
            Expr::Assign(assign) => self.print_assign(assign),
            Expr::Binary(binary) => self.print_binary(binary),
            Expr::Call(call) => self.print_call(call),
            Expr::Closure(closure) => self.print_closure(closure),
            Expr::Const(const_expr) => self.print_const_expr(const_expr),
            Expr::Field(field) => self.print_field(field),
            Expr::Class(class) => self.print_class_expr(class),
            Expr::If(if_expr) => self.print_if(if_expr),
            Expr::Let(let_expr) => self.print_let_expr(let_expr),
            Expr::Lit(lit) => self.print_literal(lit),
            Expr::MethodCall(method_call) => self.print_method_call(method_call),
            Expr::Object(object) => self.print_object_expr(object),
            Expr::Paren(paren) => self.print_paren(paren),
            Expr::Return(ret) => self.print_return(ret),
            Expr::Try(try_expr) => self.print_try(try_expr),
            Expr::TryBlock(try_block) => self.print_try_block(try_block),
            Expr::Tuple(tuple) => self.print_tuple(tuple),
            Expr::Unary(unary) => self.print_unary(unary),
            Expr::Self_ => self.print_self(),
            Expr::Super(super_expr) => self.print_super(super_expr),
            Expr::New(new_expr) => self.print_new(new_expr),
            _ => {
                self.output.push_str(&format!(
                    "{}❓ {}\n",
                    self.indent(),
                    if self.config.use_colors {
                        "Unknown Expression".red().to_string()
                    } else {
                        "Unknown Expression".to_string()
                    }
                ));
            }
        }
    }

    fn print_array(&mut self, arr: &ExprArray) {
        let array_colored = if self.config.use_colors {
            "Array".bright_blue().to_string()
        } else {
            "Array".to_string()
        };

        self.output
            .push_str(&format!("{}🔢 {}\n", self.indent(), array_colored));
        self.indent_level += 1;
        for (i, element) in arr.elements.iter().enumerate() {
            let bracket_open = if self.config.use_colors {
                "[".white().to_string()
            } else {
                "[".to_string()
            };

            let index_colored = if self.config.use_colors {
                i.to_string().cyan().to_string()
            } else {
                i.to_string()
            };

            self.output.push_str(&format!(
                "{}{}{}]:\n",
                self.indent(),
                bracket_open,
                index_colored
            ));
            self.indent_level += 1;
            self.print_expr(element);
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn print_set(&mut self, set: &ExprSet) {
        let set_colored = if self.config.use_colors {
            "Set".bright_cyan().to_string()
        } else {
            "Set".to_string()
        };

        self.output
            .push_str(&format!("{}🔹 {}\n", self.indent(), set_colored));
        self.indent_level += 1;
        for (i, element) in set.elements.iter().enumerate() {
            let brace_open = if self.config.use_colors {
                "{".white().to_string()
            } else {
                "{".to_string()
            };

            let index_colored = if self.config.use_colors {
                i.to_string().cyan().to_string()
            } else {
                i.to_string()
            };

            self.output.push_str(&format!(
                "{}{}{}:\n",
                self.indent(),
                brace_open,
                index_colored
            ));
            self.indent_level += 1;
            self.print_expr(element);
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn print_assign(&mut self, assign: &ExprAssign) {
        let assignment_colored = if self.config.use_colors {
            "Assignment".bright_yellow().to_string()
        } else {
            "Assignment".to_string()
        };

        self.output
            .push_str(&format!("{}📝 {}\n", self.indent(), assignment_colored));
        self.indent_level += 1;

        let target_colored = if self.config.use_colors {
            "Target".cyan().to_string()
        } else {
            "Target".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), target_colored));
        self.indent_level += 1;
        self.print_expr(&assign.left);
        self.indent_level -= 1;

        let value_colored = if self.config.use_colors {
            "Value".green().to_string()
        } else {
            "Value".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), value_colored));
        self.indent_level += 1;
        self.print_expr(&assign.right);
        self.indent_level -= 1;

        self.indent_level -= 1;
    }

    fn print_binary(&mut self, binary: &ExprBinary) {
        let op_str = match binary.op {
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Div => "/",
        };

        let binary_colored = if self.config.use_colors {
            "Binary Operation".bright_magenta().to_string()
        } else {
            "Binary Operation".to_string()
        };

        let op_colored = if self.config.use_colors {
            op_str.bright_red().to_string()
        } else {
            op_str.to_string()
        };

        self.output.push_str(&format!(
            "{}🔀 {} {}\n",
            self.indent(),
            binary_colored,
            op_colored
        ));

        self.indent_level += 1;

        let left_colored = if self.config.use_colors {
            "Left".cyan().to_string()
        } else {
            "Left".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), left_colored));
        self.indent_level += 1;
        self.print_expr(&binary.left);
        self.indent_level -= 1;

        let right_colored = if self.config.use_colors {
            "Right".green().to_string()
        } else {
            "Right".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), right_colored));
        self.indent_level += 1;
        self.print_expr(&binary.right);
        self.indent_level -= 1;

        self.indent_level -= 1;
    }

    fn print_call(&mut self, call: &ExprCall) {
        let call_colored = if self.config.use_colors {
            "Function Call".bright_green().to_string()
        } else {
            "Function Call".to_string()
        };

        self.output
            .push_str(&format!("{}📞 {}\n", self.indent(), call_colored));

        self.indent_level += 1;

        let callee_colored = if self.config.use_colors {
            "Callee".cyan().to_string()
        } else {
            "Callee".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), callee_colored));
        self.indent_level += 1;
        self.print_expr(&call.callee);
        self.indent_level -= 1;

        if !call.args.is_empty() {
            let args_colored = if self.config.use_colors {
                "Arguments".cyan().to_string()
            } else {
                "Arguments".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), args_colored));
            self.indent_level += 1;
            for (i, arg) in call.args.iter().enumerate() {
                let arg_label = if self.config.use_colors {
                    format!("Arg {}", i).white().to_string()
                } else {
                    format!("Arg {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), arg_label));
                self.indent_level += 1;
                self.print_expr(arg);
                self.indent_level -= 1;
            }
            self.indent_level -= 1;
        }

        self.indent_level -= 1;
    }

    fn print_closure(&mut self, closure: &ExprClosure) {
        let params_str = closure.params.join(", ");

        let closure_colored = if self.config.use_colors {
            "Closure".bright_magenta().to_string()
        } else {
            "Closure".to_string()
        };

        let params_colored = if self.config.use_colors {
            params_str.cyan().to_string()
        } else {
            params_str
        };

        let syntax = if self.config.use_colors {
            format!(
                "{{ {} => {} }}",
                params_colored,
                "...".truecolor(128, 128, 128)
            )
        } else {
            format!("{{ {} => ... }}", params_colored)
        };

        self.output.push_str(&format!(
            "{}🔗 {} {}\n",
            self.indent(),
            closure_colored,
            syntax
        ));

        self.indent_level += 1;
        self.print_expr(&closure.body);
        self.indent_level -= 1;
    }

    fn print_const_expr(&mut self, const_expr: &ExprConst) {
        let const_colored = if self.config.use_colors {
            "Const Expression".bright_magenta().to_string()
        } else {
            "Const Expression".to_string()
        };

        self.output
            .push_str(&format!("{}🔒 {}\n", self.indent(), const_colored));
        self.indent_level += 1;
        self.print_expr(&const_expr.block);
        self.indent_level -= 1;
    }

    fn print_field(&mut self, field: &ExprField) {
        let field_colored = if self.config.use_colors {
            "Field Access".bright_blue().to_string()
        } else {
            "Field Access".to_string()
        };

        let name_colored = if self.config.use_colors {
            field.name.bright_yellow().to_string()
        } else {
            field.name.clone()
        };

        self.output.push_str(&format!(
            "{}🏷️  {} {}\n",
            self.indent(),
            field_colored,
            name_colored
        ));

        self.indent_level += 1;

        let base_colored = if self.config.use_colors {
            "Base".cyan().to_string()
        } else {
            "Base".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), base_colored));
        self.indent_level += 1;
        self.print_expr(&field.base);
        self.indent_level -= 2;
    }

    fn print_class_expr(&mut self, class: &ExprClass) {
        let instance_colored = if self.config.use_colors {
            "New Instance".bright_red().to_string()
        } else {
            "New Instance".to_string()
        };

        let name_colored = if self.config.use_colors {
            class.name.bright_yellow().to_string()
        } else {
            class.name.clone()
        };

        self.output.push_str(&format!(
            "{}🏛️  {} {}\n",
            self.indent(),
            instance_colored,
            name_colored
        ));

        if !class.params.is_empty() {
            self.indent_level += 1;

            let params_colored = if self.config.use_colors {
                "Parameters".cyan().to_string()
            } else {
                "Parameters".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), params_colored));
            self.indent_level += 1;
            for (i, param) in class.params.iter().enumerate() {
                let param_label = if self.config.use_colors {
                    format!("Param {}", i).white().to_string()
                } else {
                    format!("Param {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), param_label));
                self.indent_level += 1;
                self.print_expr(param);
                self.indent_level -= 1;
            }
            self.indent_level -= 2;
        }
    }

    fn print_if(&mut self, if_expr: &ExprIf) {
        let if_colored = if self.config.use_colors {
            "If Expression".bright_yellow().to_string()
        } else {
            "If Expression".to_string()
        };

        self.output
            .push_str(&format!("{}🤔 {}\n", self.indent(), if_colored));

        self.indent_level += 1;

        let condition_colored = if self.config.use_colors {
            "Condition".cyan().to_string()
        } else {
            "Condition".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), condition_colored));
        self.indent_level += 1;
        self.print_expr(&if_expr.condition);
        self.indent_level -= 1;

        let then_colored = if self.config.use_colors {
            "Then".green().to_string()
        } else {
            "Then".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), then_colored));
        self.indent_level += 1;
        self.print_block(&if_expr.then);
        self.indent_level -= 1;

        if let Some(ref otherwise) = if_expr.otherwise {
            let else_colored = if self.config.use_colors {
                "Else".red().to_string()
            } else {
                "Else".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), else_colored));
            self.indent_level += 1;
            self.print_expr(otherwise);
            self.indent_level -= 1;
        }

        self.indent_level -= 1;
    }

    fn print_let_expr(&mut self, let_expr: &ExprLet) {
        let let_colored = if self.config.use_colors {
            "Let Expression".bright_cyan().to_string()
        } else {
            "Let Expression".to_string()
        };

        let name_colored = if self.config.use_colors {
            let_expr.name.bright_yellow().to_string()
        } else {
            let_expr.name.clone()
        };

        self.output.push_str(&format!(
            "{}🔓 {} {}\n",
            self.indent(),
            let_colored,
            name_colored
        ));
        self.indent_level += 1;
        self.print_expr(&let_expr.value);
        self.indent_level -= 1;
    }

    fn print_literal(&mut self, lit: &ExprLit) {
        let (_value_str, value_colored) = match &lit.value {
            Literal::Integer(i) => {
                let s = i.to_string();
                if self.config.use_colors {
                    (s.clone(), s.bright_blue().to_string())
                } else {
                    (s.clone(), s)
                }
            }
            Literal::Float(f) => {
                let s = f.to_string();
                if self.config.use_colors {
                    (s.clone(), s.bright_blue().to_string())
                } else {
                    (s.clone(), s)
                }
            }
            Literal::String(s) => {
                let formatted = format!("\"{}\"", s);
                if self.config.use_colors {
                    (formatted.clone(), formatted.bright_green().to_string())
                } else {
                    (formatted.clone(), formatted)
                }
            }
            Literal::Boolean(b) => {
                let s = b.to_string();
                if self.config.use_colors {
                    (s.clone(), s.bright_magenta().to_string())
                } else {
                    (s.clone(), s)
                }
            }
            Literal::Null => {
                let s = "null".to_string();
                if self.config.use_colors {
                    (s.clone(), s.truecolor(128, 128, 128).to_string())
                } else {
                    (s.clone(), s)
                }
            }
        };

        let literal_colored = if self.config.use_colors {
            "Literal".white().to_string()
        } else {
            "Literal".to_string()
        };

        self.output.push_str(&format!(
            "{}✨ {} {}\n",
            self.indent(),
            literal_colored,
            value_colored
        ));
    }

    fn print_method_call(&mut self, method_call: &ExprMethodCall) {
        let method_colored = if self.config.use_colors {
            "Method Call".bright_green().to_string()
        } else {
            "Method Call".to_string()
        };

        let name_colored = if self.config.use_colors {
            method_call.name.bright_yellow().to_string()
        } else {
            method_call.name.clone()
        };

        self.output.push_str(&format!(
            "{}📨 {} {}\n",
            self.indent(),
            method_colored,
            name_colored
        ));

        self.indent_level += 1;

        let receiver_colored = if self.config.use_colors {
            "Receiver".cyan().to_string()
        } else {
            "Receiver".to_string()
        };

        self.output
            .push_str(&format!("{}{}:\n", self.indent(), receiver_colored));
        self.indent_level += 1;
        self.print_expr(&method_call.receiver);
        self.indent_level -= 1;

        if !method_call.args.is_empty() {
            let args_colored = if self.config.use_colors {
                "Arguments".green().to_string()
            } else {
                "Arguments".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), args_colored));
            self.indent_level += 1;
            for (i, arg) in method_call.args.iter().enumerate() {
                let arg_label = if self.config.use_colors {
                    format!("Arg {}", i).white().to_string()
                } else {
                    format!("Arg {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), arg_label));
                self.indent_level += 1;
                self.print_expr(arg);
                self.indent_level -= 1;
            }
            self.indent_level -= 1;
        }

        self.indent_level -= 1;
    }

    fn print_object_expr(&mut self, object: &ExprObject) {
        let object_colored = if self.config.use_colors {
            "Object Expression".bright_red().to_string()
        } else {
            "Object Expression".to_string()
        };

        self.output
            .push_str(&format!("{}📦 {}\n", self.indent(), object_colored));

        if !object.fields.is_empty() {
            self.indent_level += 1;

            let fields_colored = if self.config.use_colors {
                "Fields".cyan().to_string()
            } else {
                "Fields".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), fields_colored));
            self.indent_level += 1;
            for (i, field) in object.fields.iter().enumerate() {
                let field_label = if self.config.use_colors {
                    format!("Field {}", i).white().to_string()
                } else {
                    format!("Field {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), field_label));
                self.indent_level += 1;
                self.print_expr(field);
                self.indent_level -= 1;
            }
            self.indent_level -= 2;
        }
    }

    fn print_paren(&mut self, paren: &ExprParen) {
        let paren_colored = if self.config.use_colors {
            "Parenthesized".white().to_string()
        } else {
            "Parenthesized".to_string()
        };

        self.output
            .push_str(&format!("{}() {}\n", self.indent(), paren_colored));
        self.indent_level += 1;
        self.print_expr(&paren.expr);
        self.indent_level -= 1;
    }

    fn print_return(&mut self, ret: &ExprReturn) {
        let return_colored = if self.config.use_colors {
            "Return".bright_red().to_string()
        } else {
            "Return".to_string()
        };

        self.output
            .push_str(&format!("{}↩️  {}\n", self.indent(), return_colored));

        if let Some(ref value) = ret.value {
            self.indent_level += 1;
            self.print_expr(value);
            self.indent_level -= 1;
        }
    }

    fn print_try(&mut self, try_expr: &ExprTry) {
        let try_colored = if self.config.use_colors {
            "Try".bright_yellow().to_string()
        } else {
            "Try".to_string()
        };

        self.output
            .push_str(&format!("{}🔥 {}\n", self.indent(), try_colored));
        self.indent_level += 1;
        self.print_expr(&try_expr.expr);
        self.indent_level -= 1;
    }

    fn print_try_block(&mut self, try_block: &ExprTryBlock) {
        let try_block_colored = if self.config.use_colors {
            "Try Block".bright_yellow().to_string()
        } else {
            "Try Block".to_string()
        };

        self.output
            .push_str(&format!("{}🛡️  {}\n", self.indent(), try_block_colored));
        self.indent_level += 1;
        self.print_block(&try_block.block);
        self.indent_level -= 1;
    }

    fn print_tuple(&mut self, tuple: &ExprTuple) {
        let tuple_colored = if self.config.use_colors {
            "Tuple".bright_cyan().to_string()
        } else {
            "Tuple".to_string()
        };

        self.output
            .push_str(&format!("{}📦 {}\n", self.indent(), tuple_colored));
        self.indent_level += 1;
        for (i, element) in tuple.elements.iter().enumerate() {
            let paren_open = if self.config.use_colors {
                "(".white().to_string()
            } else {
                "(".to_string()
            };

            let index_colored = if self.config.use_colors {
                i.to_string().cyan().to_string()
            } else {
                i.to_string()
            };

            self.output.push_str(&format!(
                "{}{}{}:\n",
                self.indent(),
                paren_open,
                index_colored
            ));
            self.indent_level += 1;
            self.print_expr(element);
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn print_unary(&mut self, unary: &ExprUnary) {
        let op_str = match unary.op {
            UnaryOp::Not => "!",
        };

        let unary_colored = if self.config.use_colors {
            "Unary Operation".bright_magenta().to_string()
        } else {
            "Unary Operation".to_string()
        };

        let op_colored = if self.config.use_colors {
            op_str.bright_red().to_string()
        } else {
            op_str.to_string()
        };

        self.output.push_str(&format!(
            "{}🔄 {} {}\n",
            self.indent(),
            unary_colored,
            op_colored
        ));

        self.indent_level += 1;
        self.print_expr(&unary.expr);
        self.indent_level -= 1;
    }

    fn print_self(&mut self) {
        let self_colored = if self.config.use_colors {
            "self".bright_blue().to_string()
        } else {
            "self".to_string()
        };

        self.output
            .push_str(&format!("{}🪞 {}\n", self.indent(), self_colored));
    }

    fn print_super(&mut self, super_expr: &ExprSuper) {
        let super_colored = if self.config.use_colors {
            "super".bright_magenta().to_string()
        } else {
            "super".to_string()
        };

        self.output
            .push_str(&format!("{}👆 {}\n", self.indent(), super_colored));

        if !super_expr.args.is_empty() {
            self.indent_level += 1;

            let args_colored = if self.config.use_colors {
                "Arguments".cyan().to_string()
            } else {
                "Arguments".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), args_colored));
            self.indent_level += 1;
            for (i, arg) in super_expr.args.iter().enumerate() {
                let arg_label = if self.config.use_colors {
                    format!("Arg {}", i).white().to_string()
                } else {
                    format!("Arg {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), arg_label));
                self.indent_level += 1;
                self.print_expr(arg);
                self.indent_level -= 1;
            }
            self.indent_level -= 2;
        }
    }

    fn print_new(&mut self, new_expr: &ExprNew) {
        let new_colored = if self.config.use_colors {
            "new".bright_red().to_string()
        } else {
            "new".to_string()
        };

        let class_colored = if self.config.use_colors {
            new_expr.class_name.bright_yellow().to_string()
        } else {
            new_expr.class_name.clone()
        };

        self.output.push_str(&format!(
            "{}🆕 {} {}\n",
            self.indent(),
            new_colored,
            class_colored
        ));

        if !new_expr.args.is_empty() {
            self.indent_level += 1;

            let args_colored = if self.config.use_colors {
                "Arguments".cyan().to_string()
            } else {
                "Arguments".to_string()
            };

            self.output
                .push_str(&format!("{}{}:\n", self.indent(), args_colored));
            self.indent_level += 1;
            for (i, arg) in new_expr.args.iter().enumerate() {
                let arg_label = if self.config.use_colors {
                    format!("Arg {}", i).white().to_string()
                } else {
                    format!("Arg {}", i)
                };

                self.output
                    .push_str(&format!("{}{}:\n", self.indent(), arg_label));
                self.indent_level += 1;
                self.print_expr(arg);
                self.indent_level -= 1;
            }
            self.indent_level -= 2;
        }
    }
}

/// Utility function to pretty print a scope with default configuration
pub fn pretty_print_scope(scope: &Scope) -> String {
    let mut printer = PrettyPrinter::new(PrettyConfig::default());
    printer.print_scope(scope)
}
