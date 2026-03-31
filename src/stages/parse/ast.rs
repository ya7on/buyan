use crate::common::{DottedPath, Spanned};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTProgram {
    pub modules: Vec<ASTModule>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTModule {
    pub name: Spanned<DottedPath>,
    pub imports: Vec<Spanned<DottedPath>>,
    pub words: Vec<Spanned<ASTWord>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTWordVar {
    Type {
        name: Spanned<String>,
        traits: Vec<Spanned<String>>,
    },
    Stack {
        name: Spanned<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTWord {
    pub name: Spanned<String>,
    pub body: Vec<Spanned<ASTInstruction>>,
    pub word_vars: Vec<ASTWordVar>,
    pub stack_effect: Spanned<ASTStackEffect>,
    pub attributes: Vec<Spanned<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ASTStackEffect {
    pub stack_in: Vec<Spanned<ASTStackEffectItem>>,
    pub stack_out: Vec<Spanned<ASTStackEffectItem>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTStackEffectItem {
    Symbol {
        name: String,
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
    String(String),
    U8(u8),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTInstruction {
    Literal(ASTLiteral),
    Call(DottedPath),
    Lambda {
        stack_effect: Spanned<ASTStackEffect>,
        body: Vec<Spanned<ASTInstruction>>,
    },
}
