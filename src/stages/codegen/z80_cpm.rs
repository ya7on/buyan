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
    fn word_label(word_id: WordId) -> String {
        format!("__word_{}", word_id.id())
    }

    fn type_size(ty: &IRType) -> usize {
        match ty {
            IRType::Bool | IRType::U8 => 1,
            IRType::U16 | IRType::Lambda => 2,
            IRType::Struct { fields } => fields.iter().map(Self::type_size).sum(),
        }
    }

    fn emit_word(ir_ctx: &IRContext, word: &IRWord) -> Result<(), DiagnosticMessage> {
        println!("; {}", word.name);
        println!("{}:", Self::word_label(word.word_id));
        for (block_id, block) in word.blocks.iter().enumerate() {
            println!("{}_bb{}:", Self::word_label(word.word_id), block_id);
            for instruction in &block.instructions {
                match &instruction.value {
                    IRInstruction::PushConstant { value } => match value {
                        IRConstant::U8(value) => {
                            println!("\tdec ix");
                            println!("\tld (ix+0), {value}");
                        }
                        IRConstant::Bool(value) => {
                            println!("\tdec ix");
                            println!("\tld (ix+0), {}", i32::from(*value));
                        }
                        IRConstant::U16(value) => {
                            println!("\tld hl, {value}");
                            println!("\tld a, h");
                            println!("\tdec ix");
                            println!("\tld (ix+0), a");
                            println!("\tld a, l");
                            println!("\tdec ix");
                            println!("\tld (ix+0), a");
                        }
                        IRConstant::StaticPtr(offset) => {
                            println!("\tld hl, __static_data+{offset}");
                            println!("\tld a, h");
                            println!("\tdec ix");
                            println!("\tld (ix+0), a");
                            println!("\tld a, l");
                            println!("\tdec ix");
                            println!("\tld (ix+0), a");
                        }
                    },
                    IRInstruction::PushLambda { word_id: _ } => todo!(),
                    IRInstruction::CallDirect { word_id } => {
                        println!("\tcall {}", Self::word_label(*word_id));
                    }
                    IRInstruction::CallIndirect => todo!(),
                    IRInstruction::Pack { type_id: _ } => {}
                    IRInstruction::Unpack { type_id: _ } => {}
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
                            println!("\tld a, (ix+{})", above_size + byte);
                            println!("\tld (ix+{}), a", destination_offset + byte);
                        }

                        if destination_offset != 0 {
                            println!("\tld de, {destination_offset}");
                            println!("\tadd ix, de");
                        }
                    }
                    IRInstruction::Load => {
                        println!("\tld e, (ix+0)");
                        println!("\tld d, (ix+1)");
                        println!("\tld a, (de)");
                        println!("\tld (ix+1), a");
                        println!("\tinc ix");
                    }
                    IRInstruction::Store => {
                        println!("\tld a, (ix+0)");
                        println!("\tld e, (ix+1)");
                        println!("\tld d, (ix+2)");
                        println!("\tld (de), a");
                        println!("\tld de, 3");
                        println!("\tadd ix, de");
                    }
                    IRInstruction::Cast { from, to } => match (from, to) {
                        (IRType::U8, IRType::U16) => {
                            println!("\tld a, (ix+0)");
                            println!("\tdec ix");
                            println!("\tld (ix+0), a");
                            println!("\tld (ix+1), 0");
                        }
                        _ => todo!(),
                    },
                    IRInstruction::Drop { ty } => {
                        let size = Self::type_size(ty);
                        println!("\tld de, {size}");
                        println!("\tadd ix, de");
                    }
                    IRInstruction::Dup { ty: _ } => todo!(),
                    IRInstruction::Swap { lower, upper } => {
                        let lower_size = Self::type_size(lower);
                        let upper_size = Self::type_size(upper);

                        for byte in 0..upper_size {
                            println!("\tld a, (ix+{byte})");
                            println!("\tpush af");
                        }

                        for byte in 0..lower_size {
                            println!("\tld a, (ix+{})", upper_size + byte);
                            println!("\tld (ix+{byte}), a");
                        }

                        for byte in (0..upper_size).rev() {
                            println!("\tpop af");
                            println!("\tld (ix+{}), a", lower_size + byte);
                        }
                    }
                    IRInstruction::Over { lower, upper } => {
                        let lower_size = Self::type_size(lower);
                        let upper_size = Self::type_size(upper);

                        println!("\tld de, -{lower_size}");
                        println!("\tadd ix, de");

                        let source_offset = lower_size + upper_size;
                        for byte in 0..lower_size {
                            println!("\tld a, (ix+{})", source_offset + byte);
                            println!("\tld (ix+{}), a", byte);
                        }
                    }
                    IRInstruction::Add { ty } => match ty {
                        IRType::U8 => {
                            println!("\tld a, (ix+0)");
                            println!("\tinc ix");
                            println!("\tadd a, (ix+0)");
                            println!("\tld (ix+0), a");
                        }
                        IRType::U16 => {
                            println!("\tld a, (ix+0)");
                            println!("\tld e, a");
                            println!("\tld a, (ix+1)");
                            println!("\tld d, a");
                            println!("\tld a, (ix+2)");
                            println!("\tld l, a");
                            println!("\tld a, (ix+3)");
                            println!("\tld h, a");
                            println!("\tadd hl, de");
                            println!("\tld a, l");
                            println!("\tld (ix+2), a");
                            println!("\tld a, h");
                            println!("\tld (ix+3), a");
                            println!("\tld de, 2");
                            println!("\tadd ix, de");
                        }
                        _ => todo!(),
                    },
                    IRInstruction::Sub { ty: _ } => todo!(),
                    IRInstruction::Mul { ty: _ } => todo!(),
                    IRInstruction::Div { ty: _ } => todo!(),
                    IRInstruction::Eq { ty: _ } => todo!(),
                    IRInstruction::Gt { ty: _ } => todo!(),
                    IRInstruction::Lt { ty } => match ty {
                        IRType::U8 => {
                            println!("\tld a, (ix+1)");
                            println!("\tsub (ix+0)");
                            println!("\tld a, 0");
                            println!("\tsbc a, a");
                            println!("\tand 1");
                            println!("\tld (ix+1), a");
                            println!("\tinc ix");
                        }
                        _ => todo!(),
                    },
                    IRInstruction::PutChar => {
                        println!("\tld e, (ix+0)");
                        println!("\tinc ix");
                        println!("\tld c, 2");
                        println!("\tcall 5");
                    }
                }
                println!(" ");
            }

            match block.terminator.value {
                IRTerminator::End => {
                    println!("\tret");
                }
                IRTerminator::Branch { branch } => {
                    println!("\tjp {}_bb{}", Self::word_label(word.word_id), branch.0);
                }
                IRTerminator::BranchIfZero {
                    then_branch,
                    else_branch,
                } => {
                    println!("\tld a, (ix+0)");
                    println!("\tinc ix");
                    println!("\tor a");
                    println!(
                        "\tjp nz, {}_bb{}",
                        Self::word_label(word.word_id),
                        then_branch.0
                    );
                    println!(
                        "\tjp {}_bb{}",
                        Self::word_label(word.word_id),
                        else_branch.0
                    );
                }
            }
        }

        Ok(())
    }
}

impl Stage<CompileContext> for Z80CpmCodegenStage {
    type Input = (IRContext, IRProgram);
    type Output = ();

    fn execute(
        &mut self,
        (ir_ctx, ir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let Some(entrypoint) = ir_program.words.iter().find(|word| word.entrypoint) else {
            return StageResult::success(());
        };

        println!("ORG 0x100");
        println!("start:");
        println!("\tld ix, __data_stack_end");
        println!("\tcall {}", Self::word_label(entrypoint.word_id));
        println!("\tld c, 0");
        println!("\tcall 5");

        let mut diagnostics = Diagnostics::default();
        for word in &ir_program.words {
            if let Err(error) = Self::emit_word(&ir_ctx, word) {
                diagnostics.emit_fatal(error);
                return StageResult::new(None, diagnostics);
            }
        }

        println!("__static_data:");
        for byte in &ir_program.static_data {
            println!("\tdb {byte}");
        }
        println!("__data_stack:");
        println!("\tdefs 256");
        println!("__data_stack_end:");

        StageResult::new(Some(()), diagnostics)
    }
}
