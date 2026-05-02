use crate::{
    common::CompileContext,
    error::CompileError,
    pipeline::Stage,
    stages::lower::{
        context::{IRContext, WordId},
        ir::{IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRValue {
    U8(u8),
    String(String),
    Lambda(WordId),
}

impl IRValue {
    fn from_constant(constant: &IRConstant) -> Self {
        match constant {
            IRConstant::U8(value) => Self::U8(*value),
            IRConstant::String(value) => Self::String(value.clone()),
        }
    }
}

#[derive(Debug, Default)]
pub struct IRInterpreter {
    stack: Vec<IRValue>,
}

#[allow(dead_code)]
impl IRInterpreter {
    pub fn stack(&self) -> &[IRValue] {
        &self.stack
    }

    pub fn execute(&mut self, program: &IRProgram) {
        self.stack.clear();

        let word_id = program
            .words
            .iter()
            .position(|word| word.entrypoint)
            .expect("word not found");

        self.execute_word(program, WordId(word_id));
    }

    fn execute_word(&mut self, program: &IRProgram, word_id: WordId) {
        let word = program.words.get(word_id.id()).expect("word not found");
        let block = word.blocks.first().expect("word has no blocks");

        self.execute_block(program, block);
    }

    fn execute_block(&mut self, program: &IRProgram, block: &IRBasicBlock) {
        for instruction in &block.instructions {
            println!("{:?}", instruction);
            match &instruction.value {
                IRInstruction::PushConstant { value } => {
                    self.stack.push(IRValue::from_constant(value));
                }
                IRInstruction::PushLambda { word_id } => {
                    self.stack.push(IRValue::Lambda(*word_id));
                }
                IRInstruction::CallDirect { word_id } => {
                    self.execute_word(program, *word_id);
                }
                IRInstruction::CallIndirect => {
                    let lambda = self.stack.pop().expect("stack underflow");
                    match lambda {
                        IRValue::Lambda(word_id) => self.execute_word(program, word_id),
                        _ => panic!("indirect call expects lambda"),
                    }
                }
                IRInstruction::Swap => {
                    let rhs = self.stack.pop().expect("stack underflow");
                    let lhs = self.stack.pop().expect("stack underflow");
                    self.stack.push(rhs);
                    self.stack.push(lhs);
                }
                IRInstruction::Dup => {
                    let value = self.stack.last().cloned().expect("stack underflow");
                    self.stack.push(value);
                }
                IRInstruction::Drop => {
                    self.stack.pop().expect("stack underflow");
                }
                IRInstruction::Add => {
                    let rhs = self.stack.pop().expect("stack underflow");
                    let lhs = self.stack.pop().expect("stack underflow");
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs + rhs));
                        }
                        _ => todo!(),
                    }
                }
            }
            println!("{:?}", self.stack);
        }

        match &block.terminator.value {
            IRTerminator::End => {}
            IRTerminator::Branch { .. } => todo!(),
            IRTerminator::BranchIfZero { .. } => todo!(),
        }
    }
}

impl Stage<CompileContext> for IRInterpreter {
    type Input = (IRContext, IRProgram);
    type Output = ();

    fn execute(
        &mut self,
        (_ir_ctx, ir_program): Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        IRInterpreter::execute(self, &ir_program);
        Ok(())
    }
}
