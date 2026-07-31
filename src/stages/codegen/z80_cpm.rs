use crate::{
    common::CompileContext,
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

    fn emit_word(word: &IRWord) {
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
                    },
                    IRInstruction::Add { ty: IRType::U8 } => {
                        println!("\tld a, (ix+0)");
                        println!("\tinc ix");
                        println!("\tadd a, (ix+0)");
                        println!("\tld (ix+0), a");
                    }
                    IRInstruction::CallDirect { word_id } => {
                        println!("\tcall {}", Self::word_label(*word_id));
                    }
                    _ => {
                        println!("; UNSUPPORTED INSTRUCTION");
                    }
                }
                println!(" ");
            }

            match block.terminator.value {
                IRTerminator::End => {
                    println!("\tret");
                }
                _ => {
                    println!("; UNSUPPORTED TERMINATOR");
                }
            }
        }
    }
}

impl Stage<CompileContext> for Z80CpmCodegenStage {
    type Input = (IRContext, IRProgram);
    type Output = ();

    fn execute(
        &mut self,
        (_ir_ctx, ir_program): Self::Input,
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

        for word in &ir_program.words {
            Self::emit_word(word);
        }

        println!("__data_stack:");
        println!("\tdefs 256");
        println!("__data_stack_end:");

        StageResult::success(())
    }
}
