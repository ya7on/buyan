use crate::{
    common::{CompileContext, DottedPath, Spanned},
    error::CompileError,
    pipeline::Stage,
    stages::semantic::{
        context::{HIRContext, SymbolKind},
        hir::{HIRInstruction, HIRLiteral, HIRProgram, HIRType, HIRWord, HIRWordAttribute},
        stack_analysis::StackAnalysis,
    },
};

#[derive(Default)]
pub struct TypeCheckStage;

impl TypeCheckStage {
    fn type_check_instruction(
        hir_ctx: &HIRContext,
        instruction: &Spanned<HIRInstruction>,
        stack_analysis: &mut StackAnalysis,
    ) -> Result<(), CompileError> {
        match &instruction.value {
            HIRInstruction::Call { name, symbol_id } => {
                let Some(SymbolKind::Word {
                    typevars: _,
                    stackvars: _,
                    stack_in,
                    stack_out,
                }) = hir_ctx.get(*symbol_id)
                else {
                    return Err(CompileError::SymbolNotFound {
                        name: name.to_string(),
                        span: instruction.span,
                    });
                };
                stack_analysis.apply_call(
                    stack_in.iter().map(|item| &item.value).cloned().collect(),
                    stack_out.iter().map(|item| &item.value).cloned().collect(),
                    instruction.span,
                )?;
            }
            HIRInstruction::Literal(literal) => match literal {
                HIRLiteral::U8(_) => {
                    let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse("u8")) else {
                        return Err(CompileError::SymbolNotFound {
                            name: "u8".to_string(),
                            span: instruction.span,
                        });
                    };
                    stack_analysis.push(HIRType::BuiltIn(symbol_id));
                }
                HIRLiteral::String(_) => {
                    let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse("string")) else {
                        return Err(CompileError::SymbolNotFound {
                            name: "string".to_string(),
                            span: instruction.span,
                        });
                    };
                    stack_analysis.push(HIRType::BuiltIn(symbol_id));
                }
            },
            HIRInstruction::Lambda {
                stack_in,
                stack_out,
                body,
            } => {
                stack_analysis.push(HIRType::Lambda {
                    stack_in: stack_in.clone(),
                    stack_out: stack_out.clone(),
                });

                let mut lambda_stack_analysis = StackAnalysis::new(stack_in.clone());

                for instruction in body {
                    TypeCheckStage::type_check_instruction(
                        hir_ctx,
                        instruction,
                        &mut lambda_stack_analysis,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn type_check_word(word: &HIRWord, hir_ctx: &HIRContext) -> Result<(), CompileError> {
        let mut stack_analysis = StackAnalysis::new(
            word.signature
                .stack_in
                .iter()
                .map(|item| &item.value)
                .cloned()
                .collect(),
        );

        for instruction in &word.body {
            TypeCheckStage::type_check_instruction(hir_ctx, instruction, &mut stack_analysis)?;
        }

        stack_analysis.match_stack(
            word.signature
                .stack_out
                .iter()
                .map(|item| &item.value)
                .cloned()
                .collect(),
            word.signature.name.span, // TODO use stack_out span
        )?;

        Ok(())
    }
}

impl Stage<CompileContext> for TypeCheckStage {
    type Input = (HIRContext, HIRProgram);
    type Output = (HIRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut errors = Vec::new();
        for module in &hir_program.modules {
            for word in &module.words {
                if word.attributes.contains(&HIRWordAttribute::BuiltIn) {
                    continue;
                }

                if let Err(err) = TypeCheckStage::type_check_word(word, &hir_ctx) {
                    errors.push(err);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((hir_ctx, hir_program))
    }
}
