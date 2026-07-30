use std::io::Write;

use crate::{
    common::CompileContext,
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::lower::{
        context::{IRContext, TypeId, WordId},
        ir::{BasicBlockId, IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRValue {
    Bool(bool),
    U8(u8),
    Lambda(WordId),
    Struct { type_id: TypeId, fields: Vec<Self> },
    Array(Vec<Self>),
}

impl IRValue {
    const fn from_constant(constant: &IRConstant) -> Self {
        match constant {
            IRConstant::Bool(value) => Self::Bool(*value),
            IRConstant::U8(value) => Self::U8(*value),
        }
    }
}

#[derive(Debug, Default)]
pub struct IRInterpreter {
    stack: Vec<IRValue>,
}

#[allow(dead_code)]
impl IRInterpreter {
    #[must_use]
    pub fn stack(&self) -> &[IRValue] {
        &self.stack
    }

    fn execute_program(&mut self, program: &IRProgram) -> Result<(), DiagnosticMessage> {
        self.stack.clear();

        let word_id = program
            .words
            .iter()
            .position(|word| word.entrypoint)
            .ok_or(DiagnosticMessage::RuntimeError("word not found"))?;

        self.execute_word(program, WordId(word_id))
    }

    fn execute_word(
        &mut self,
        program: &IRProgram,
        word_id: WordId,
    ) -> Result<(), DiagnosticMessage> {
        let word = program
            .words
            .get(word_id.id())
            .ok_or(DiagnosticMessage::RuntimeError("word not found"))?;
        let mut block_id = BasicBlockId(0);
        loop {
            let block = word
                .blocks
                .get(block_id.0)
                .ok_or(DiagnosticMessage::RuntimeError("block not found"))?;
            let Some(next_block_id) = self.execute_block(program, block)? else {
                break;
            };
            block_id = next_block_id;
        }
        Ok(())
    }

    fn execute_block(
        &mut self,
        program: &IRProgram,
        block: &IRBasicBlock,
    ) -> Result<Option<BasicBlockId>, DiagnosticMessage> {
        for instruction in &block.instructions {
            match &instruction.value {
                IRInstruction::PushConstant { value } => {
                    self.stack.push(IRValue::from_constant(value));
                }
                IRInstruction::PushLambda { word_id } => {
                    self.stack.push(IRValue::Lambda(*word_id));
                }
                IRInstruction::CallDirect { word_id } => {
                    self.execute_word(program, *word_id)?;
                }
                IRInstruction::CallIndirect => {
                    let lambda = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match lambda {
                        IRValue::Lambda(word_id) => self.execute_word(program, word_id)?,
                        _ => panic!("indirect call expects lambda"),
                    }
                }
                IRInstruction::Pack {
                    type_id,
                    field_count,
                } => {
                    let start = self
                        .stack
                        .len()
                        .checked_sub(*field_count)
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let fields = self.stack.split_off(start);
                    self.stack.push(IRValue::Struct {
                        type_id: *type_id,
                        fields,
                    });
                }
                IRInstruction::PackArray { element_count } => {
                    let start = self
                        .stack
                        .len()
                        .checked_sub(*element_count)
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let elements = self.stack.split_off(start);
                    self.stack.push(IRValue::Array(elements));
                }
                IRInstruction::Unpack { type_id } => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::Struct {
                        type_id: actual_type_id,
                        fields,
                    } = value
                    else {
                        panic!("unpack expects struct");
                    };
                    assert_eq!(actual_type_id, *type_id, "struct type mismatch");
                    self.stack.extend(fields);
                }
                IRInstruction::GetField {
                    type_id,
                    field_index,
                } => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::Struct {
                        type_id: actual_type_id,
                        fields,
                    } = value
                    else {
                        panic!("get field expects struct");
                    };
                    assert_eq!(actual_type_id, *type_id, "struct type mismatch");
                    let field = fields
                        .into_iter()
                        .nth(*field_index)
                        .ok_or(DiagnosticMessage::RuntimeError("field index out of bounds"))?;
                    self.stack.push(field);
                }
                IRInstruction::ArrayIndex => {
                    let index = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let array = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::U8(index) = index else {
                        panic!("array index expects u8 index");
                    };
                    let IRValue::Array(elements) = array else {
                        panic!("array index expects array");
                    };
                    let element = elements
                        .into_iter()
                        .nth(usize::from(index))
                        .ok_or(DiagnosticMessage::RuntimeError("array index out of bounds"))?;
                    self.stack.push(element);
                }
                IRInstruction::Swap { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    self.stack.push(rhs);
                    self.stack.push(lhs);
                }
                IRInstruction::Dup { .. } => {
                    let value = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    self.stack.push(value);
                }
                IRInstruction::Drop { .. } => {
                    self.stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                }
                IRInstruction::Add { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs + rhs));
                        }
                        _ => todo!(),
                    }
                }
                IRInstruction::Sub { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs - rhs));
                        }
                        _ => todo!(),
                    }
                }
                IRInstruction::Mul { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs * rhs));
                        }
                        _ => todo!(),
                    }
                }
                IRInstruction::Div { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs / rhs));
                        }
                        _ => todo!(),
                    }
                }
                IRInstruction::Eq { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::Bool(lhs == rhs));
                        }
                        (IRValue::Bool(lhs), IRValue::Bool(rhs)) => {
                            self.stack.push(IRValue::Bool(lhs == rhs));
                        }
                        _ => todo!(),
                    }
                }
                IRInstruction::Gt { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::Bool(lhs > rhs));
                        }
                        _ => panic!("gt expects u8 operands"),
                    }
                }
                IRInstruction::Lt { .. } => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (lhs, rhs) {
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::Bool(lhs < rhs));
                        }
                        _ => panic!("lt expects u8 operands"),
                    }
                }
                IRInstruction::Print => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match value {
                        IRValue::Array(values) => {
                            let bytes = values
                                .into_iter()
                                .map(|value| match value {
                                    IRValue::U8(value) => Ok(value),
                                    _ => Err(DiagnosticMessage::RuntimeError(
                                        "print expects an array of u8",
                                    )),
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            std::io::stdout().write_all(&bytes).map_err(|_| {
                                DiagnosticMessage::RuntimeError("failed to write output")
                            })?;
                        }
                        IRValue::U8(value) => {
                            print!("{value}");
                        }
                        IRValue::Bool(value) => {
                            print!("{value}");
                        }
                        _ => {
                            print!("{value:?}");
                        }
                    }
                }
                IRInstruction::Flush => {
                    std::io::stdout()
                        .flush()
                        .map_err(|_| DiagnosticMessage::RuntimeError("failed to flush stdout"))?;
                }
            }
        }

        let next_block = match &block.terminator.value {
            IRTerminator::End => None,
            IRTerminator::Branch { branch } => Some(*branch),
            IRTerminator::BranchIfZero {
                then_branch,
                else_branch,
            } => {
                let else_lambda = self
                    .stack
                    .pop()
                    .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                let then_lambda = self
                    .stack
                    .pop()
                    .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                let condition = self
                    .stack
                    .pop()
                    .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;

                let IRValue::Bool(condition) = condition else {
                    panic!("branch expects bool condition");
                };

                if condition {
                    self.stack.push(then_lambda);
                    Some(*then_branch)
                } else {
                    self.stack.push(else_lambda);
                    Some(*else_branch)
                }
            }
        };
        Ok(next_block)
    }
}

impl Stage<CompileContext> for IRInterpreter {
    type Input = (IRContext, IRProgram);
    type Output = ();

    fn execute(
        &mut self,
        (_ir_ctx, ir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        match self.execute_program(&ir_program) {
            Ok(()) => StageResult::new(Some(()), diagnostics),
            Err(error) => {
                diagnostics.emit_fatal(error);
                StageResult::new(None, diagnostics)
            }
        }
    }
}
