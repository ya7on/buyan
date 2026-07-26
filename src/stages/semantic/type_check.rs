use crate::{
    common::{CompileContext, DottedPath, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
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
        stack_analysis: &mut StackAnalysis<'_>,
    ) -> Result<(), DiagnosticMessage> {
        match &instruction.value {
            HIRInstruction::Call { name, symbol_id } => {
                let Some(SymbolKind::Word {
                    typevars: _,
                    stackvars: _,
                    stack_in,
                    stack_out,
                }) = hir_ctx.get(*symbol_id)
                else {
                    return Err(DiagnosticMessage::SymbolNotFound {
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
                HIRLiteral::Bool(_) => {
                    let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse("bool")) else {
                        return Err(DiagnosticMessage::SymbolNotFound {
                            name: "bool".to_string(),
                            span: instruction.span,
                        });
                    };
                    stack_analysis.push(HIRType::BuiltIn(symbol_id));
                }
                HIRLiteral::U8(_) => {
                    let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse("u8")) else {
                        return Err(DiagnosticMessage::SymbolNotFound {
                            name: "u8".to_string(),
                            span: instruction.span,
                        });
                    };
                    stack_analysis.push(HIRType::BuiltIn(symbol_id));
                }
                HIRLiteral::String(_) => {
                    let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse("string")) else {
                        return Err(DiagnosticMessage::SymbolNotFound {
                            name: "string".to_string(),
                            span: instruction.span,
                        });
                    };
                    stack_analysis.push(HIRType::BuiltIn(symbol_id));
                }
            },
            HIRInstruction::Pack { name, struct_id } => {
                let Some(SymbolKind::Struct { fields, .. }) = hir_ctx.get(*struct_id) else {
                    return Err(DiagnosticMessage::SymbolNotFound {
                        name: name.clone(),
                        span: instruction.span,
                    });
                };
                stack_analysis.apply_call(
                    fields.iter().map(|item| item.value.clone()).collect(),
                    vec![HIRType::Struct(*struct_id)],
                    instruction.span,
                )?;
            }
            HIRInstruction::Unpack { name, struct_id } => {
                let Some(SymbolKind::Struct { fields, .. }) = hir_ctx.get(*struct_id) else {
                    return Err(DiagnosticMessage::SymbolNotFound {
                        name: name.clone(),
                        span: instruction.span,
                    });
                };
                stack_analysis.apply_call(
                    vec![HIRType::Struct(*struct_id)],
                    fields.iter().map(|item| item.value.clone()).collect(),
                    instruction.span,
                )?;
            }
            HIRInstruction::GetField {
                name,
                struct_id,
                field_index,
            } => {
                let Some(SymbolKind::Struct { fields, .. }) = hir_ctx.get(*struct_id) else {
                    return Err(DiagnosticMessage::SymbolNotFound {
                        name: name.clone(),
                        span: instruction.span,
                    });
                };
                let Some(field) = fields.get(*field_index) else {
                    return Err(DiagnosticMessage::InvalidFieldIndex {
                        name: name.clone(),
                        index: *field_index,
                        field_count: fields.len(),
                        span: instruction.span,
                    });
                };
                stack_analysis.apply_call(
                    vec![HIRType::Struct(*struct_id)],
                    vec![field.value.clone()],
                    instruction.span,
                )?;
            }
            HIRInstruction::Lambda {
                stack_in,
                stack_out,
                body,
            } => {
                stack_analysis.push(HIRType::Lambda {
                    stack_in: stack_in.clone(),
                    stack_out: stack_out.clone(),
                });

                let mut lambda_stack_analysis = StackAnalysis::new(hir_ctx, stack_in.clone());

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

    fn type_check_word(word: &HIRWord, hir_ctx: &HIRContext) -> Result<(), DiagnosticMessage> {
        let mut stack_analysis = StackAnalysis::new(
            hir_ctx,
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
            word.signature.stack_effect_span,
            word.body
                .last()
                .map(|instruction| instruction.span)
                .into_iter()
                .collect(),
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
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        for module in &hir_program.modules {
            for word in &module.words {
                if word.attributes.contains(&HIRWordAttribute::BuiltIn) {
                    continue;
                }

                if let Err(err) = TypeCheckStage::type_check_word(word, &hir_ctx) {
                    diagnostics.emit_fatal(err);
                }
            }
        }

        StageResult::new(Some((hir_ctx, hir_program)), diagnostics)
    }
}
