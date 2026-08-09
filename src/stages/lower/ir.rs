use crate::{
    common::{CompileTarget, Spanned},
    stages::lower::context::{TypeId, WordId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicBlockId(pub usize);

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub words: Vec<Spanned<IRWord>>,
    pub types: Vec<IRType>,
    pub static_data: Vec<u8>,
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
    U16(u16),
    StaticPtr(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRType {
    Bool,
    U8,
    U16,
    Struct { fields: Vec<Self> },
    Lambda,
}

impl IRType {
    pub(crate) fn from_builtin_type(target: CompileTarget, name: &str) -> Option<Self> {
        match name {
            "bool" => Some(Self::Bool),
            "u8" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "ptr" | "usize" => match target {
                CompileTarget::Interpreter | CompileTarget::Z80UnknownCpm => Some(Self::U16),
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    PushConstant { value: IRConstant },
    PushLambda { word_id: WordId },
    CallDirect { word_id: WordId },
    CallExtern { symbol: String },
    CallIndirect,
    Pack { type_id: TypeId },
    Unpack { type_id: TypeId },
    GetField { type_id: TypeId, field_index: usize },
    Load,
    Store,
    Cast { from: IRType, to: IRType },
    Drop { ty: IRType },
    Dup { ty: IRType },
    Swap { lower: IRType, upper: IRType },
    Over { lower: IRType, upper: IRType },
    Add { ty: IRType },
    Sub { ty: IRType },
    Mul { ty: IRType },
    Div { ty: IRType },
    Eq { ty: IRType },
    Gt { ty: IRType },
    Lt { ty: IRType },
    PutChar,
}
