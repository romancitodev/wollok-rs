use crate::expr::{Block, Expr};
use owo_colors::OwoColorize;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Item {
    Const(ItemConst),
    Let(ItemLet),
    Property(ItemProperty),
    Method(ItemMethod),
    PrefixedMethod(ItemPrefixedMethod),
    Class(ItemClass),
    Object(ItemObject),
    Import(ItemImport),
    Test(ItemTest),
    Program(ItemProgram),
    Package(ItemPackage),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemConst {
    pub name: String,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemLet {
    pub name: String,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemProperty {
    pub name: String,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub ident: String,
    pub params: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemMethod {
    pub signature: Signature,
    pub body: Block,
    pub inline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Prefix {
    Override,
    Fallible,
    OverrideFallible,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemPrefixedMethod {
    pub prefix: Prefix,
    pub method: ItemMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemClass {
    pub name: String,
    pub superclass: Option<Vec<String>>,
    pub body: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemObject {
    pub name: String,
    pub body: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemImport {
    pub module: String,
    pub wildcard: bool, // true para "import modulo.*", false para imports específicos
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemTest {
    pub name: String,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemProgram {
    pub name: String,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemPackage {
    pub name: String,
    pub body: Vec<Item>,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Const(item) => write!(f, "{item}"),
            Item::Let(item) => write!(f, "{item}"),
            Item::Property(item) => write!(f, "{item}"),
            Item::Method(item) => write!(f, "{item}"),
            Item::Class(item) => write!(f, "{item}"),
            Item::Object(item) => write!(f, "{item}"),
            Item::Import(item) => write!(f, "{item}"),
            Item::Test(item) => write!(f, "{item}"),
            Item::Program(item) => write!(f, "{item}"),
            Item::Package(item) => write!(f, "{item}"),
            Item::PrefixedMethod(item) => write!(f, "{item}"),
        }
    }
}

impl Display for ItemConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            "(item) ".yellow(),
            "const ".magenta(),
            self.name.cyan(),
            " = ".white(),
            self.expr
        )
    }
}

impl Display for ItemLet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            "(item) ".yellow(),
            "let ".magenta(),
            self.name.cyan(),
            " = ".white(),
            self.expr
        )
    }
}

impl Display for ItemProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            "(item) ".yellow(),
            "property ".magenta(),
            self.name.cyan(),
            " = ".white(),
            self.expr
        )
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.cyan())
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident.blue())?;
        write!(f, "(")?;
        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{param}")?;
            if i < self.params.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for ItemMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inline {
            write!(
                f,
                "{}{} = {}",
                "method ".magenta(),
                self.signature,
                self.body
                    .stmts
                    .first()
                    .expect("Method body should have at least one statement")
            )
        } else {
            write!(f, "{}{} {}", "method ".magenta(), self.signature, self.body)
        }
    }
}

impl Display for ItemClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "class ".magenta(), self.name.cyan())?;
        if let Some(superclass) = &self.superclass {
            write!(f, "{}", " inherits ".magenta())?;
            for class in superclass {
                write!(f, "{}, ", class.cyan())?;
            }
        }
        write!(f, "{{")?;
        for item in &self.body {
            write!(f, " {item}; ")?;
        }
        write!(f, " }}")
    }
}

impl Display for ItemObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "object ".magenta(), self.name.cyan())?;
        writeln!(f, " {{")?;
        for item in &self.body {
            writeln!(f, "\t {item}; ")?;
        }
        writeln!(f, " }}")
    }
}

impl Display for ItemImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "import ".magenta(), self.module.cyan())?;
        if self.wildcard {
            write!(f, ".*")?;
        }
        Ok(())
    }
}

impl Display for ItemTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{} {}", "test ".magenta(), self.name.cyan(), self.body)
    }
}

impl Display for ItemProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} {}",
            "program ".magenta(),
            self.name.cyan(),
            self.body
        )
    }
}

impl Display for ItemPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "package ".magenta(), self.name.cyan())?;
        writeln!(f, " {{")?;
        for item in &self.body {
            write!(f, " {item}; ")?;
        }
        writeln!(f, " }}")
    }
}
impl Display for ItemPrefixedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prefix {
            Prefix::Override => write!(f, "{}", "override ".magenta()),
            Prefix::Fallible => write!(f, "{}", "fallible ".magenta()),
            Prefix::OverrideFallible => write!(f, "{}", "override fallible ".magenta()),
        }?;
        write!(f, "{}", self.method)
    }
}
