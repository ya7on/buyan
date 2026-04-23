use crate::{
    common::{CompileContext, Spanned},
    error::CompileError,
    pipeline::Stage,
    stages::{
        lower::{
            context::{IRContext, WordId},
            ir::{IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator, IRWord},
        },
        semantic::hir::{HIRInstruction, HIRLiteral, HIRModule, HIRProgram, HIRWord},
    },
};

#[derive(Default)]
pub struct BasicBlockBuilder {
    pub instructions: Vec<Spanned<IRInstruction>>,
}

impl BasicBlockBuilder {
    pub fn push(&mut self, instruction: Spanned<IRInstruction>) {
        self.instructions.push(instruction);
    }

    pub fn build(self, terminator: Spanned<IRTerminator>) -> IRBasicBlock {
        IRBasicBlock {
            instructions: self.instructions,
            terminator,
        }
    }
}

#[derive(Default)]
pub struct LowerStage;

impl LowerStage {
    fn lower_module(module: &HIRModule) -> Result<Vec<Spanned<IRWord>>, Vec<CompileError>> {
        let mut errors = Vec::new();
        let mut result = Vec::new();

        for word in &module.words {
            let word = match Self::lower_word(word) {
                Ok(word) => word,
                Err(err) => {
                    errors.extend(err);
                    continue;
                }
            };
            result.push(word);
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(result)
    }

    fn lower_word(word: &Spanned<HIRWord>) -> Result<Spanned<IRWord>, Vec<CompileError>> {
        let errors = Vec::new();
        let mut result = IRWord {
            blocks: Vec::new(),
            entrypoint: word.entrypoint,
        };

        let mut basicblock = BasicBlockBuilder::default();
        for instruction in &word.body {
            match &instruction.value {
                HIRInstruction::Call { name, .. } => {
                    match name.as_str() {
                        // Builtin call
                        "std.prelude.if" => {
                            todo!()
                        }
                        "std.stack.call" => {
                            basicblock
                                .push(Spanned::new(IRInstruction::CallLambda, instruction.span));
                        }
                        "std.stack.drop" => {
                            basicblock.push(Spanned::new(IRInstruction::Drop, instruction.span));
                        }
                        "std.stack.dup" => {
                            basicblock.push(Spanned::new(IRInstruction::Dup, instruction.span));
                        }
                        "std.stack.swap" => {
                            basicblock.push(Spanned::new(IRInstruction::Swap, instruction.span));
                        }
                        "std.math.add" => {
                            basicblock.push(Spanned::new(IRInstruction::Add, instruction.span));
                        }
                        // Real word call
                        _ => {
                            basicblock.push(Spanned::new(
                                IRInstruction::Call { word_id: WordId(0) },
                                instruction.span,
                            ));
                        }
                    }
                }
                HIRInstruction::Literal(literal) => match literal {
                    HIRLiteral::U8(value) => {
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant {
                                value: IRConstant::U8(*value),
                            },
                            instruction.span,
                        ));
                    }
                    HIRLiteral::String(value) => {
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant {
                                value: IRConstant::String(value.to_string()),
                            },
                            instruction.span,
                        ));
                    }
                },
                HIRInstruction::Lambda {
                    stack_in: _,
                    stack_out: _,
                    body: _,
                } => {
                    // TODO
                }
            }
        }
        result
            .blocks
            .push(basicblock.build(Spanned::new(IRTerminator::End, word.span)));

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Spanned::new(result, word.span))
    }
}

impl Stage<CompileContext> for LowerStage {
    type Input = (IRContext, HIRProgram);
    type Output = (IRContext, IRProgram);

    fn execute(
        &mut self,
        (ir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut errors = Vec::new();
        let mut result = IRProgram { words: Vec::new() };

        for module in &hir_program.modules {
            let words = match Self::lower_module(module) {
                Ok(words) => words,
                Err(err) => {
                    errors.extend(err);
                    continue;
                }
            };
            result.words.extend(words);
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((ir_ctx, result))
    }
}
