use crate::{common::Spanned, stages::lower::context::WordId};

#[derive(Debug, Clone)]
pub struct BasicBlockId(pub usize);

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub words: Vec<Spanned<IRWord>>,
}

#[derive(Debug, Clone)]
pub struct IRWord {
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
    U8(u8),
    String(String),
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    PushConstant {
        value: IRConstant,
    },
    Call {
        word_id: WordId,
    },
    If {
        then_branch: BasicBlockId,
        else_branch: BasicBlockId,
    },
    While {
        condition: BasicBlockId,
        body: BasicBlockId,
    },
    CallLambda,
    Drop,
    Dup,
    Swap,
    Add,
}
