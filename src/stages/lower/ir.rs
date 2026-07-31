use crate::{
    common::Spanned,
    stages::lower::context::{TypeId, WordId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicBlockId(pub usize);

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub words: Vec<Spanned<IRWord>>,
}

#[derive(Debug, Clone)]
pub struct IRWord {
    pub word_id: WordId,
    pub name: String,
    pub entrypoint: bool,
    pub blocks: Vec<IRBasicBlock>,
}

#[derive(Debug, Clone)]
pub struct IRBasicBlock {
    pub instructions: Vec<Spanned<IRInstruction>>,
    pub terminator: Spanned<IRTerminator>,
}

#[derive(Debug, Clone)]
pub enum IRTerminator {
    Branch {
        branch: BasicBlockId,
    },
    BranchIfZero {
        then_branch: BasicBlockId,
        else_branch: BasicBlockId,
    },
    End,
}

#[derive(Debug, Clone)]
pub enum IRConstant {
    Bool(bool),
    U8(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRType {
    Bool,
    U8,
    Struct {
        fields: Vec<Self>,
    },
    Array {
        element_type: Box<Self>,
        size: usize,
    },
    Lambda,
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    PushConstant { value: IRConstant },
    PushLambda { word_id: WordId },
    CallDirect { word_id: WordId },
    CallIndirect,
    Pack { type_id: TypeId, field_count: usize },
    Unpack { type_id: TypeId },
    GetField { type_id: TypeId, field_index: usize },
    PackArray { element_count: usize },
    ArrayIndex,
    Drop { ty: IRType },
    Dup { ty: IRType },
    Swap { lower: IRType, upper: IRType },
    Add { ty: IRType },
    Sub { ty: IRType },
    Mul { ty: IRType },
    Div { ty: IRType },
    Eq { ty: IRType },
    Gt { ty: IRType },
    Lt { ty: IRType },
    Print { ty: IRType },
}
