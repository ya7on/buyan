use crate::{common::Spanned, stages::lower::context::WordId};

#[derive(Debug)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct IRProgram {
    pub words: Vec<Spanned<IRWord>>,
}

#[derive(Debug)]
pub struct IRWord {
    pub entrypoint: bool,
    pub blocks: Vec<IRBasicBlock>,
}

#[derive(Debug)]
pub struct IRBasicBlock {
    pub instructions: Vec<Spanned<IRInstruction>>,
    pub terminator: Spanned<IRTerminator>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum IRConstant {
    U8(u8),
    String(String),
}

#[derive(Debug)]
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
