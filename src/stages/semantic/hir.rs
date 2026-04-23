use std::collections::HashMap;

use crate::{common::Spanned, stages::semantic::context::SymbolId};

#[derive(Debug, Clone)]
pub struct HIRProgram {
    pub modules: Vec<HIRModule>,
}

#[derive(Debug, Clone)]
pub struct HIRModule {
    pub id: SymbolId,
    pub imports: Vec<Spanned<SymbolId>>,
    pub words: Vec<Spanned<HIRWord>>,
}

#[derive(Debug, Clone)]
pub struct HIRWordSignature {
    pub name: Spanned<String>,
    pub stack_in: Vec<Spanned<HIRType>>,
    pub stack_out: Vec<Spanned<HIRType>>,
    pub type_vars: Vec<Spanned<SymbolId>>,
    pub stack_vars: Vec<Spanned<SymbolId>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HIRWordAttribute {
    BuiltIn,
}

#[derive(Debug, Clone)]
pub struct HIRWord {
    pub id: SymbolId,
    pub signature: HIRWordSignature,
    pub body: Vec<Spanned<HIRInstruction>>,
    pub attributes: Vec<HIRWordAttribute>,
    pub entrypoint: bool,
    pub substitutions: HashMap<SymbolId, SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HIRType {
    BuiltIn(SymbolId),
    TypeVar(SymbolId),
    StackVar(SymbolId),
    Lambda {
        stack_in: Vec<HIRType>,
        stack_out: Vec<HIRType>,
    },
}

#[derive(Debug, Clone)]
pub enum HIRLiteral {
    U8(u8),
    String(String),
}

#[derive(Debug, Clone)]
pub enum HIRInstruction {
    Call {
        name: String,
        symbol_id: SymbolId,
        // substitutions: HashMap<SymbolId, SymbolId>,
    },
    Literal(HIRLiteral),
    Lambda {
        stack_in: Vec<HIRType>,
        stack_out: Vec<HIRType>,
        body: Vec<Spanned<HIRInstruction>>,
    },
}
