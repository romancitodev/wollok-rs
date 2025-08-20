use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Item {
    Const(ItemConst),
    Let(ItemLet),
    Property(ItemProperty),
    Method(ItemMethod),
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
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemMethod {
    pub signature: Signature,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemClass {
    pub name: String,
    pub superclass: Option<String>,
    pub body: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemObject {
    pub name: Option<String>, // Los objetos pueden ser anónimos
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
