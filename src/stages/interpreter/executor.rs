use std::io::{Read, Write};

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
            IRConstant::StaticPtr(address) => Self::U16(*address),
        }
    }
}

#[derive(Debug)]
pub struct IRInterpreter {
    stack: Vec<IRValue>,
    memory: Vec<u8>,
    heap_ptr: usize,
}

impl Default for IRInterpreter {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            memory: vec![0; usize::from(u16::MAX) + 1],
            heap_ptr: 0,
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
        self.heap_ptr = program.static_data.len();

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
                IRInstruction::CallExtern { symbol } => match symbol.as_str() {
                    "put_char" => {
                        let value = self
                            .stack
                            .pop()
                            .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                        let IRValue::U8(value) = value else {
                            unreachable!("invalid IR type for interpreter");
                        };
                        std::io::stdout().write_all(&[value]).map_err(|_| {
                            DiagnosticMessage::RuntimeError("failed to write output")
                        })?;
                    }
                    "read_char" => {
                        let mut value = [0];
                        std::io::stdin()
                            .read_exact(&mut value)
                            .map_err(|_| DiagnosticMessage::RuntimeError("failed to read input"))?;
                        self.stack.push(IRValue::U8(value[0]));
                    }
                    "alloc" => {
                        let size = self
                            .stack
                            .pop()
                            .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                        let IRValue::U16(size) = size else {
                            unreachable!("invalid IR type for interpreter");
                        };
                        let address = u16::try_from(self.heap_ptr)
                            .ok()
                            .filter(|_| {
                                self.heap_ptr
                                    .checked_add(usize::from(size))
                                    .is_some_and(|end| end <= self.memory.len())
                            })
                            .ok_or(DiagnosticMessage::RuntimeError("out of memory"))?;
                        self.heap_ptr += usize::from(size);
                        self.stack.push(IRValue::U16(address));
                    }
                    _ => {
                        return Err(DiagnosticMessage::RuntimeError(
                            "cannot execute an external word in interpreter",
                        ));
                    }
                },
                IRInstruction::CallIndirect => {
                    let lambda = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    match lambda {
                        IRValue::Lambda(word_id) => self.execute_word(program, word_id)?,
                        _ => unreachable!("invalid IR type for interpreter"),
                    }
                }
                IRInstruction::Pack { type_id } => {
                    let Some(IRType::Struct { fields }) = program.types.get(type_id.id()) else {
                        unreachable!("invalid IR type for interpreter");
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
                        unreachable!("invalid IR type for interpreter");
                    };
                    if actual_type_id != *type_id {
                        unreachable!("invalid IR type for interpreter");
                    }
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
                        unreachable!("invalid IR type for interpreter");
                    };
                    if actual_type_id != *type_id {
                        unreachable!("invalid IR type for interpreter");
                    }
                    let Some(field) = fields.into_iter().nth(*field_index) else {
                        unreachable!("invalid IR type for interpreter");
                    };
                    self.stack.push(field);
                }
                IRInstruction::Load => {
                    let address = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let IRValue::U16(address) = address else {
                        unreachable!("invalid IR type for interpreter");
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
                        unreachable!("invalid IR type for interpreter");
                    };
                    let IRValue::U16(address) = address else {
                        unreachable!("invalid IR type for interpreter");
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
                        (IRType::U16, IRType::U8, IRValue::U16(value)) => {
                            self.stack.push(IRValue::U8(value as u8));
                        }
                        _ => unreachable!("invalid IR type for interpreter"),
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
                IRInstruction::RotateLeft { .. } => {
                    let upper = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let middle = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lower = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    self.stack.push(middle);
                    self.stack.push(upper);
                    self.stack.push(lower);
                }
                IRInstruction::RotateRight { .. } => {
                    let upper = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let middle = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    let lower = self
                        .stack
                        .pop()
                        .ok_or(DiagnosticMessage::RuntimeError("stack underflow"))?;
                    self.stack.push(upper);
                    self.stack.push(lower);
                    self.stack.push(middle);
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
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        (IRValue::U8(_), IRValue::U8(0)) | (IRValue::U16(_), IRValue::U16(0)) => {
                            return Err(DiagnosticMessage::RuntimeError("division by zero"));
                        }
                        (IRValue::U8(lhs), IRValue::U8(rhs)) => {
                            self.stack.push(IRValue::U8(lhs / rhs));
                        }
                        (IRValue::U16(lhs), IRValue::U16(rhs)) => {
                            self.stack.push(IRValue::U16(lhs / rhs));
                        }
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        _ => unreachable!("invalid IR type for interpreter"),
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
                        _ => unreachable!("invalid IR type for interpreter"),
                    }
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
                    unreachable!("invalid IR type for interpreter");
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
