use crate::{
    common::{CompileContext, Span, Spanned},
    error::CompileError,
    pipeline::Stage,
    stages::{
        lower::{
            context::{IRContext, WordId},
            ir::{IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator, IRWord},
        },
        semantic::hir::{HIRInstruction, HIRLiteral, HIRProgram, HIRWord},
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
    fn lower_word(
        ir_ctx: &IRContext,
        word: &Spanned<HIRWord>,
        lambda_words: &mut Vec<Spanned<IRWord>>,
        base_word_count: usize,
    ) -> Result<Spanned<IRWord>, Vec<CompileError>> {
        Ok(Spanned::new(
            Self::lower_ir_word(
                ir_ctx,
                &word.body,
                word.entrypoint,
                word.span,
                lambda_words,
                base_word_count,
            )?,
            word.span,
        ))
    }

    fn lower_ir_word(
        ir_ctx: &IRContext,
        body: &[Spanned<HIRInstruction>],
        entrypoint: bool,
        span: Span,
        lambda_words: &mut Vec<Spanned<IRWord>>,
        base_word_count: usize,
    ) -> Result<IRWord, Vec<CompileError>> {
        let mut errors = Vec::new();
        let mut basicblock = BasicBlockBuilder::default();
        for instruction in body {
            match &instruction.value {
                HIRInstruction::Call { name, symbol_id } => {
                    match name.as_str() {
                        // Builtin call
                        "std.prelude.if" => {
                            todo!()
                        }
                        "std.stack.call" => {
                            basicblock
                                .push(Spanned::new(IRInstruction::CallIndirect, instruction.span));
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
                            let Some(word_id) = ir_ctx.symbol_id_to_word_id.get(symbol_id).copied() else {
                                errors.push(CompileError::SymbolNotFound {
                                    name: name.clone(),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::CallDirect { word_id },
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
                    body,
                } => {
                    let lambda_slot = lambda_words.len();
                    let word_id = WordId(base_word_count + lambda_slot);

                    // Reserve the slot before lowering nested lambdas so this id stays stable.
                    lambda_words.push(Spanned::new(
                        IRWord {
                            entrypoint: false,
                            blocks: Vec::new(),
                        },
                        instruction.span,
                    ));

                    let lambda_word = match Self::lower_ir_word(
                        ir_ctx,
                        body,
                        false,
                        instruction.span,
                        lambda_words,
                        base_word_count,
                    ) {
                        Ok(word) => word,
                        Err(err) => {
                            errors.extend(err);
                            continue;
                        }
                    };

                    lambda_words[lambda_slot] = Spanned::new(lambda_word, instruction.span);
                    basicblock.push(Spanned::new(
                        IRInstruction::PushLambda { word_id },
                        instruction.span,
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(IRWord {
            entrypoint,
            blocks: vec![basicblock.build(Spanned::new(IRTerminator::End, span))],
        })
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
        let mut lambda_words = Vec::new();
        let base_word_count = ir_ctx.words.len();

        for module in &hir_program.modules {
            for word in &module.words {
                match Self::lower_word(&ir_ctx, word, &mut lambda_words, base_word_count) {
                    Ok(word) => result.words.push(word),
                    Err(err) => {
                        errors.extend(err);
                    }
                }
            }
        }
        result.words.extend(lambda_words);

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((ir_ctx, result))
    }
}
