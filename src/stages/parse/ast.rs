use crate::common::{DottedPath, Spanned};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTProgram {
    pub modules: Vec<ASTModule>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTModule {
    pub name: Spanned<DottedPath>,
    pub imports: Vec<Spanned<ASTImport>>,
    pub structs: Vec<Spanned<ASTStruct>>,
    pub words: Vec<Spanned<ASTWord>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTImport {
    pub name: DottedPath,
    pub attributes: Vec<Spanned<ASTAttribute>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTStruct {
    pub name: Spanned<String>,
    pub fields: Vec<Spanned<ASTStackEffectItem>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTWordVar {
    Type { name: Spanned<String> },
    Stack { name: Spanned<String> },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTWord {
    pub name: Spanned<String>,
    pub body: Vec<Spanned<ASTInstruction>>,
    pub word_vars: Vec<ASTWordVar>,
    pub stack_effect: Spanned<ASTStackEffect>,
    pub attributes: Vec<Spanned<ASTAttribute>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTAttribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTStackEffect {
    pub stack_in: Vec<Spanned<ASTStackEffectItem>>,
    pub stack_out: Vec<Spanned<ASTStackEffectItem>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTStackEffectItem {
    Symbol {
        name: DottedPath,
    },
    StackVar {
        name: String,
    },
    Lambda {
        stack_effect: Spanned<ASTStackEffect>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTLiteral {
    Bool(bool),
    String(String),
    U8(u8),
    U16(u16),
    Usize(u16),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTInstruction {
    Literal(ASTLiteral),
    Call(DottedPath),
    Pack(DottedPath),
    Unpack(DottedPath),
    Lambda {
        stack_effect: Spanned<ASTStackEffect>,
        body: Vec<Spanned<Self>>,
    },
}
