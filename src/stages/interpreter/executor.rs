use std::io::Write;

use crate::{
    common::CompileContext,
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::lower::{
        context::{IRContext, TypeId, WordId},
        ir::{
            BasicBlockId, IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator, IRType,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    Lambda(WordId),
    Struct { type_id: TypeId, fields: Vec<Self> },
}

impl IRValue {
    const fn from_constant(constant: &IRConstant) -> Self {
        match constant {
            IRConstant::Bool(value) => Self::Bool(*value),
            IRConstant::U8(value) => Self::U8(*value),
            IRConstant::U16(value) => Self::U16(*value),
        }
    }
}

#[derive(Debug)]
pub struct IRInterpreter {
    stack: Vec<IRValue>,
    memory: Vec<u8>,
}

impl Default for IRInterpreter {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            memory: vec![0; usize::from(u16::MAX) + 1],
        }
    }
}

#[allow(dead_code)]
impl IRInterpreter {
    #[must_use]
    pub fn stack(&self) -> &[IRValue] {
        &self.stack
    }

    fn execute_program(&mut self, program: &IRProgram) -> Result<(), DiagnosticMessage> {
        self.stack.clear();
        self.memory.fill(0);
        self.memory[..program.static_data.len()].copy_from_slice(&program.static_data);

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
                        _ => {
                            return Err(DiagnosticMessage::RuntimeError(
                                "indirect call expects lambda",
                            ));
                        }
                    }
                }
                IRInstruction::Pack { type_id } => {
                    let Some(IRType::Struct { fields }) = program.types.get(type_id.id()) else {
                        return Err(DiagnosticMessage::RuntimeError("pack expects struct type"));
                    };
                    let start = self
                        .stack
                        .len()
                        .checked_sub(fields.len())
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let fields = self.stack.split_off(start);
                    self.stack.push(IRValue::Struct {
                        type_id: *type_id,
                        fields,
                    });
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
                IRInstruction::Load => {
                    let address = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::U16(address) = address else {
                        panic!("load expects ptr");
                    };
                    let value = *self.memory.get(usize::from(address)).ok_or(
                        DiagnosticMessage::RuntimeError("memory address out of bounds"),
                    )?;
                    self.stack.push(IRValue::U8(value));
                }
                IRInstruction::Store => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let address = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::U8(value) = value else {
                        panic!("store expects u8 value");
                    };
                    let IRValue::U16(address) = address else {
                        panic!("store expects ptr");
                    };
                    let slot = self.memory.get_mut(usize::from(address)).ok_or(
                        DiagnosticMessage::RuntimeError("memory address out of bounds"),
                    )?;
                    *slot = value;
                }
                IRInstruction::Cast { from, to } => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match (from, to, value) {
                        (IRType::U8, IRType::U16, IRValue::U8(value)) => {
                            self.stack.push(IRValue::U16(u16::from(value)));
                        }
                        _ => return Err(DiagnosticMessage::RuntimeError("invalid cast")),
                    }
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
                IRInstruction::Over { .. } => {
                    let upper = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lower = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    self.stack.push(lower.clone());
                    self.stack.push(upper);
                    self.stack.push(lower);
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
                            self.stack.push(IRValue::U8(lhs.wrapping_add(rhs)));
                        }
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::U16(lhs.wrapping_add(rhs)));
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
                            self.stack.push(IRValue::U8(lhs.wrapping_sub(rhs)));
                        }
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::U16(lhs.wrapping_sub(rhs)));
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
                            self.stack.push(IRValue::U8(lhs.wrapping_mul(rhs)));
                        }
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::U16(lhs.wrapping_mul(rhs)));
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
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::U16(lhs / rhs));
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
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
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
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
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
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::Bool(lhs < rhs));
                        }
                        _ => panic!("lt expects u8 operands"),
                    }
                }
                IRInstruction::PutChar => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::U8(value) = value else {
                        panic!("put char expects u8");
                    };
                    std::io::stdout()
                        .write_all(&[value])
                        .map_err(|_| DiagnosticMessage::RuntimeError("failed to write output"))?;
                }
            }

            #[cfg(debug_assertions)]
            eprintln!(
                "\x1b[38;5;240m[interpreter] {:?} => {:?}\x1b[0m",
                instruction.value, self.stack
            );
        }

        let next_block = match &block.terminator.value {
            IRTerminator::End => None,
            IRTerminator::Branch { branch } => Some(*branch),
            IRTerminator::BranchIfZero {
                then_branch,
                else_branch,
            } => {
                let condition = self
                    .stack
                    .pop()
                    .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;

                let IRValue::Bool(condition) = condition else {
                    return Err(DiagnosticMessage::RuntimeError(
                        "branch expects bool condition",
                    ));
                };

                if condition {
                    Some(*then_branch)
                } else {
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
