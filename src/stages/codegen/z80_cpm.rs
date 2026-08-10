use super::assembler::{
    emitter::Emitter,
    z80::{
        assembly::Z80Assembly,
        builder::Z80,
        condition::Z80Condition,
        operand::{Z80Immediate, Z80Label, Z80Operand},
        register::Z80Register,
    },
};
use crate::{
    common::CompileContext,
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::lower::{
        context::{IRContext, WordId},
        ir::{IRConstant, IRInstruction, IRProgram, IRTerminator, IRType, IRWord},
    },
};

#[derive(Debug)]
pub struct Z80CpmCodegenStage;

impl Z80CpmCodegenStage {
    fn word_label(word_id: WordId) -> Z80Label {
        Z80Label::new(format!("__word_{}", word_id.id()))
    }

    fn type_size(ty: &IRType) -> usize {
        match ty {
            IRType::Bool | IRType::U8 => 1,
            IRType::U16 | IRType::Lambda => 2,
            IRType::Struct { fields } => fields.iter().map(Self::type_size).sum(),
        }
    }

    fn block_label(word_id: WordId, block_id: usize) -> Z80Label {
        Z80Label::new(format!("{}_bb{block_id}", Self::word_label(word_id)))
    }

    fn emit_word(
        emitter: &mut Emitter<Z80Assembly>,
        ir_ctx: &IRContext,
        word: &IRWord,
    ) -> Result<(), DiagnosticMessage> {
        emitter.emit(Z80::comment(&word.name));
        emitter.emit(Z80::label(Self::word_label(word.word_id)));
        for (block_id, block) in word.blocks.iter().enumerate() {
            emitter.emit(Z80::label(Self::block_label(word.word_id, block_id)));
            for (instruction_index, instruction) in block.instructions.iter().enumerate() {
                match &instruction.value {
                    IRInstruction::PushConstant { value } => match value {
                        IRConstant::U8(value) => {
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Immediate((*value).into()),
                            ));
                        }
                        IRConstant::Bool(value) => {
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Immediate(isize::from(*value)),
                            ));
                        }
                        IRConstant::U16(value) => {
                            emitter.emit(Z80::ld(Z80Register::HL, Z80Immediate(*value as isize)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::H));
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::L));
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                        }
                        IRConstant::StaticPtr(offset) => {
                            emitter.emit(Z80::ld(
                                Z80Register::HL,
                                Z80Operand::label_offset(
                                    Z80Label::new("__static_data"),
                                    *offset as isize,
                                ),
                            ));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::H));
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::L));
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                        }
                    },
                    IRInstruction::PushLambda { word_id: _ } => todo!(),
                    IRInstruction::CallDirect { word_id } => {
                        emitter.emit(Z80::call(Self::word_label(*word_id)));
                    }
                    IRInstruction::CallExtern { symbol } => {
                        let mut characters = symbol.bytes();
                        if !matches!(characters.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
                            || !characters.all(|character| {
                                matches!(character, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
                            })
                        {
                            return Err(DiagnosticMessage::Unknown {
                                label: format!("invalid Z80 external symbol '{symbol}'"),
                            });
                        }
                        emitter.emit(Z80::call(Z80Label::new(symbol.clone())));
                    }
                    IRInstruction::CallIndirect => todo!(),
                    IRInstruction::Pack { type_id: _ } | IRInstruction::Unpack { type_id: _ } => {}
                    IRInstruction::GetField {
                        type_id,
                        field_index,
                    } => {
                        let Some(IRType::Struct { fields }) = ir_ctx.get_type(*type_id) else {
                            return Err(DiagnosticMessage::Unknown {
                                label: format!(
                                    "GetField references a non-struct IR type with id {}",
                                    type_id.id()
                                ),
                            });
                        };
                        let Some(field) = fields.get(*field_index) else {
                            return Err(DiagnosticMessage::Unknown {
                                label: format!(
                                    "GetField references field {field_index} of a struct with {} fields",
                                    fields.len()
                                ),
                            });
                        };

                        let field_size = Self::type_size(field);
                        let above_size: usize =
                            fields[*field_index + 1..].iter().map(Self::type_size).sum();
                        let struct_size: usize = fields.iter().map(Self::type_size).sum();

                        let destination_offset = struct_size - field_size;
                        for byte in (0..field_size).rev() {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, (above_size + byte) as isize),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(
                                    Z80Register::IX,
                                    (destination_offset + byte) as isize,
                                ),
                                Z80Register::A,
                            ));
                        }

                        if destination_offset != 0 {
                            emitter.emit(Z80::ld(
                                Z80Register::DE,
                                Z80Immediate(destination_offset as isize),
                            ));
                            emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                        }
                    }
                    IRInstruction::Load => {
                        emitter.emit(Z80::ld(
                            Z80Register::E,
                            Z80Operand::indexed(Z80Register::IX, 0),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Register::D,
                            Z80Operand::indexed(Z80Register::IX, 1),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Register::A,
                            Z80Operand::indirect(Z80Register::DE),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Operand::indexed(Z80Register::IX, 1),
                            Z80Register::A,
                        ));
                        emitter.emit(Z80::inc(Z80Register::IX));
                    }
                    IRInstruction::Store => {
                        emitter.emit(Z80::ld(
                            Z80Register::A,
                            Z80Operand::indexed(Z80Register::IX, 0),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Register::E,
                            Z80Operand::indexed(Z80Register::IX, 1),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Register::D,
                            Z80Operand::indexed(Z80Register::IX, 2),
                        ));
                        emitter.emit(Z80::ld(
                            Z80Operand::indirect(Z80Register::DE),
                            Z80Register::A,
                        ));
                        emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(3)));
                        emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                    }
                    IRInstruction::Cast { from, to } => match (from, to) {
                        (IRType::U8, IRType::U16) => {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 0),
                            ));
                            emitter.emit(Z80::dec(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 1),
                                Z80Immediate(0),
                            ));
                        }
                        (IRType::U16, IRType::U8) => {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 0),
                            ));
                            emitter.emit(Z80::inc(Z80Register::IX));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                        }
                        _ => todo!(),
                    },
                    IRInstruction::Drop { ty } => {
                        let size = Self::type_size(ty);
                        emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(size as isize)));
                        emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                    }
                    IRInstruction::Dup { ty } => {
                        let size = Self::type_size(ty);

                        emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(-(size as isize))));
                        emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));

                        for byte in 0..size {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, (size + byte) as isize),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                                Z80Register::A,
                            ));
                        }
                    }
                    IRInstruction::Swap { lower, upper } => {
                        let lower_size = Self::type_size(lower);
                        let upper_size = Self::type_size(upper);

                        for byte in 0..upper_size {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                            ));
                            emitter.emit(Z80::push(Z80Register::AF));
                        }

                        for byte in 0..lower_size {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, (upper_size + byte) as isize),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                                Z80Register::A,
                            ));
                        }

                        for byte in (0..upper_size).rev() {
                            emitter.emit(Z80::pop(Z80Register::AF));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, (lower_size + byte) as isize),
                                Z80Register::A,
                            ));
                        }
                    }
                    IRInstruction::Over { lower, upper } => {
                        let lower_size = Self::type_size(lower);
                        let upper_size = Self::type_size(upper);

                        emitter.emit(Z80::ld(
                            Z80Register::DE,
                            Z80Immediate(-(lower_size as isize)),
                        ));
                        emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));

                        let source_offset = lower_size + upper_size;
                        for byte in 0..lower_size {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(
                                    Z80Register::IX,
                                    (source_offset + byte) as isize,
                                ),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                                Z80Register::A,
                            ));
                        }
                    }
                    IRInstruction::RotateLeft {
                        lower,
                        middle,
                        upper,
                    } => {
                        let lower_size = Self::type_size(lower);
                        let middle_size = Self::type_size(middle);
                        let upper_size = Self::type_size(upper);
                        let total_size = lower_size + middle_size + upper_size;
                        let segments = [
                            (middle_size + upper_size, lower_size),
                            (0, upper_size),
                            (upper_size, middle_size),
                        ];

                        for (offset, size) in segments.into_iter().rev() {
                            for byte in (0..size).rev() {
                                emitter.emit(Z80::ld(
                                    Z80Register::A,
                                    Z80Operand::indexed(Z80Register::IX, (offset + byte) as isize),
                                ));
                                emitter.emit(Z80::push(Z80Register::AF));
                            }
                        }

                        for byte in 0..total_size {
                            emitter.emit(Z80::pop(Z80Register::AF));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                                Z80Register::A,
                            ));
                        }
                    }
                    IRInstruction::RotateRight {
                        lower,
                        middle,
                        upper,
                    } => {
                        let lower_size = Self::type_size(lower);
                        let middle_size = Self::type_size(middle);
                        let upper_size = Self::type_size(upper);
                        let total_size = lower_size + middle_size + upper_size;
                        let segments = [
                            (upper_size, middle_size),
                            (middle_size + upper_size, lower_size),
                            (0, upper_size),
                        ];

                        for (offset, size) in segments.into_iter().rev() {
                            for byte in (0..size).rev() {
                                emitter.emit(Z80::ld(
                                    Z80Register::A,
                                    Z80Operand::indexed(Z80Register::IX, (offset + byte) as isize),
                                ));
                                emitter.emit(Z80::push(Z80Register::AF));
                            }
                        }

                        for byte in 0..total_size {
                            emitter.emit(Z80::pop(Z80Register::AF));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, byte as isize),
                                Z80Register::A,
                            ));
                        }
                    }
                    IRInstruction::Add { ty } => match ty {
                        IRType::U8 => {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 0),
                            ));
                            emitter.emit(Z80::inc(Z80Register::IX));
                            emitter.emit(Z80::add(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 0),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 0),
                                Z80Register::A,
                            ));
                        }
                        IRType::U16 => {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 0),
                            ));
                            emitter.emit(Z80::ld(Z80Register::E, Z80Register::A));
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 1),
                            ));
                            emitter.emit(Z80::ld(Z80Register::D, Z80Register::A));
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 2),
                            ));
                            emitter.emit(Z80::ld(Z80Register::L, Z80Register::A));
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 3),
                            ));
                            emitter.emit(Z80::ld(Z80Register::H, Z80Register::A));
                            emitter.emit(Z80::add(Z80Register::HL, Z80Register::DE));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::L));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 2),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Register::H));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 3),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(2)));
                            emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                        }
                        _ => todo!(),
                    },
                    IRInstruction::Sub { ty: _ } => todo!(),
                    IRInstruction::Mul { ty: _ } => todo!(),
                    IRInstruction::Div { ty: _ } => todo!(),
                    IRInstruction::Eq { ty } => match ty {
                        IRType::U8 => {
                            let not_equal_label = format!(
                                "{}_bb{block_id}_eq_u8_{instruction_index}_not_equal",
                                Self::word_label(word.word_id),
                            );
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 1),
                            ));
                            emitter.emit(Z80::cp(Z80Operand::indexed(Z80Register::IX, 0)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Immediate(0)));
                            emitter.emit(Z80::jr(
                                Z80Condition::Nz,
                                Z80Label::new(not_equal_label.clone()),
                            ));
                            emitter.emit(Z80::inc(Z80Register::A));
                            emitter.emit(Z80::label(Z80Label::new(not_equal_label)));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 1),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::inc(Z80Register::IX));
                        }
                        IRType::U16 => {
                            let not_equal_label = format!(
                                "{}_bb{block_id}_eq_u16_{instruction_index}_not_equal",
                                Self::word_label(word.word_id),
                            );
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 3),
                            ));
                            emitter.emit(Z80::cp(Z80Operand::indexed(Z80Register::IX, 1)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Immediate(0)));
                            emitter.emit(Z80::jr(
                                Z80Condition::Nz,
                                Z80Label::new(not_equal_label.clone()),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 2),
                            ));
                            emitter.emit(Z80::cp(Z80Operand::indexed(Z80Register::IX, 0)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Immediate(0)));
                            emitter.emit(Z80::jr(
                                Z80Condition::Nz,
                                Z80Label::new(not_equal_label.clone()),
                            ));
                            emitter.emit(Z80::inc(Z80Register::A));
                            emitter.emit(Z80::label(Z80Label::new(not_equal_label)));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 3),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(3)));
                            emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                        }
                        _ => todo!(),
                    },
                    IRInstruction::Gt { ty: _ } => todo!(),
                    IRInstruction::Lt { ty } => match ty {
                        IRType::U8 => {
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 1),
                            ));
                            emitter.emit(Z80::sub(Z80Operand::indexed(Z80Register::IX, 0)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Immediate(0)));
                            emitter.emit(Z80::sbc(Z80Register::A, Z80Register::A));
                            emitter.emit(Z80::and(Z80Immediate(1)));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 1),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::inc(Z80Register::IX));
                        }
                        IRType::U16 => {
                            let compare_done_label = format!(
                                "{}_bb{block_id}_lt_u16_{instruction_index}_compare_done",
                                Self::word_label(word.word_id),
                            );
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 3),
                            ));
                            emitter.emit(Z80::cp(Z80Operand::indexed(Z80Register::IX, 1)));
                            emitter.emit(Z80::jr(
                                Z80Condition::Nz,
                                Z80Label::new(compare_done_label.clone()),
                            ));
                            emitter.emit(Z80::ld(
                                Z80Register::A,
                                Z80Operand::indexed(Z80Register::IX, 2),
                            ));
                            emitter.emit(Z80::cp(Z80Operand::indexed(Z80Register::IX, 0)));
                            emitter.emit(Z80::label(Z80Label::new(compare_done_label)));
                            emitter.emit(Z80::ld(Z80Register::A, Z80Immediate(0)));
                            emitter.emit(Z80::sbc(Z80Register::A, Z80Register::A));
                            emitter.emit(Z80::and(Z80Immediate(1)));
                            emitter.emit(Z80::ld(
                                Z80Operand::indexed(Z80Register::IX, 3),
                                Z80Register::A,
                            ));
                            emitter.emit(Z80::ld(Z80Register::DE, Z80Immediate(3)));
                            emitter.emit(Z80::add(Z80Register::IX, Z80Register::DE));
                        }
                        _ => todo!(),
                    },
                }
            }

            match block.terminator.value {
                IRTerminator::End => {
                    emitter.emit(Z80::ret());
                }
                IRTerminator::Branch { branch } => {
                    emitter.emit(Z80::jp(None, Self::block_label(word.word_id, branch.0)));
                }
                IRTerminator::BranchIfZero {
                    then_branch,
                    else_branch,
                } => {
                    emitter.emit(Z80::ld(
                        Z80Register::A,
                        Z80Operand::indexed(Z80Register::IX, 0),
                    ));
                    emitter.emit(Z80::inc(Z80Register::IX));
                    emitter.emit(Z80::or(Z80Register::A));
                    emitter.emit(Z80::jp(
                        Some(Z80Condition::Nz),
                        Self::block_label(word.word_id, then_branch.0),
                    ));
                    emitter.emit(Z80::jp(
                        None,
                        Self::block_label(word.word_id, else_branch.0),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Stage<CompileContext> for Z80CpmCodegenStage {
    type Input = (IRContext, IRProgram);
    type Output = String;

    fn execute(
        &mut self,
        (ir_ctx, ir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let Some(entrypoint) = ir_program.words.iter().find(|word| word.entrypoint) else {
            return StageResult::success(String::new());
        };

        let mut emitter = Emitter::<Z80Assembly>::default();
        emitter.emit(Z80::org(0x100));
        emitter.emit(Z80::label(Z80Label::new("start")));
        emitter.emit(Z80::ld(Z80Register::HL, Z80Label::new("__heap")));
        emitter.emit(Z80::ld(
            Z80Operand::indirect_label(Z80Label::new("__heap_ptr")),
            Z80Register::HL,
        ));
        emitter.emit(Z80::ld(Z80Register::IX, Z80Label::new("__data_stack_end")));
        emitter.emit(Z80::call(Self::word_label(entrypoint.word_id)));
        emitter.emit(Z80::ld(Z80Register::C, Z80Immediate(0)));
        emitter.emit(Z80::call(Z80Immediate(5)));

        emitter.emit(Z80::label(Z80Label::new("bdos_call")));
        emitter.emit(Z80::ld(
            Z80Register::C,
            Z80Operand::indexed(Z80Register::IX, 0),
        ));
        emitter.emit(Z80::ld(
            Z80Register::E,
            Z80Operand::indexed(Z80Register::IX, 1),
        ));
        emitter.emit(Z80::ld(
            Z80Register::D,
            Z80Operand::indexed(Z80Register::IX, 2),
        ));
        emitter.emit(Z80::push(Z80Register::IX));
        emitter.emit(Z80::call(Z80Immediate(5)));
        emitter.emit(Z80::pop(Z80Register::IX));
        emitter.emit(Z80::ld(
            Z80Operand::indexed(Z80Register::IX, 1),
            Z80Register::L,
        ));
        emitter.emit(Z80::ld(
            Z80Operand::indexed(Z80Register::IX, 2),
            Z80Register::H,
        ));
        emitter.emit(Z80::inc(Z80Register::IX));
        emitter.emit(Z80::ret());

        emitter.emit(Z80::label(Z80Label::new("alloc")));
        emitter.emit(Z80::ld(
            Z80Register::E,
            Z80Operand::indexed(Z80Register::IX, 0),
        ));
        emitter.emit(Z80::ld(
            Z80Register::D,
            Z80Operand::indexed(Z80Register::IX, 1),
        ));
        emitter.emit(Z80::ld(
            Z80Register::HL,
            Z80Operand::indirect_label(Z80Label::new("__heap_ptr")),
        ));
        emitter.emit(Z80::push(Z80Register::HL));
        emitter.emit(Z80::add(Z80Register::HL, Z80Register::DE));
        emitter.emit(Z80::ld(
            Z80Operand::indirect_label(Z80Label::new("__heap_ptr")),
            Z80Register::HL,
        ));
        emitter.emit(Z80::pop(Z80Register::HL));
        emitter.emit(Z80::ld(
            Z80Operand::indexed(Z80Register::IX, 0),
            Z80Register::L,
        ));
        emitter.emit(Z80::ld(
            Z80Operand::indexed(Z80Register::IX, 1),
            Z80Register::H,
        ));

        emitter.emit(Z80::ret());

        let mut diagnostics = Diagnostics::default();
        for word in &ir_program.words {
            if let Err(error) = Self::emit_word(&mut emitter, &ir_ctx, word) {
                diagnostics.emit_fatal(error);
                return StageResult::new(None, diagnostics);
            }
        }

        emitter.emit(Z80::label(Z80Label::new("__static_data")));
        for byte in &ir_program.static_data {
            emitter.emit(Z80::db(*byte));
        }
        emitter.emit(Z80::label(Z80Label::new("__data_stack")));
        emitter.emit(Z80::defs(256));
        emitter.emit(Z80::label(Z80Label::new("__data_stack_end")));
        emitter.emit(Z80::label(Z80Label::new("__heap_ptr")));
        emitter.emit(Z80::defs(2));
        emitter.emit(Z80::label(Z80Label::new("__heap")));

        StageResult::new(Some(emitter.finish()), diagnostics)
    }
}
