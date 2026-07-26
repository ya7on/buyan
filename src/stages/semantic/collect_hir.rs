use std::collections::HashMap;

use crate::{
    common::{CompileContext, DottedPath, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        parse::ast::{ASTInstruction, ASTLiteral, ASTModule, ASTProgram, ASTStruct, ASTWord},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::{
                HIRInstruction, HIRLiteral, HIRModule, HIRProgram, HIRStruct, HIRWord,
                HIRWordAttribute, HIRWordSignature,
            },
        },
    },
};

#[derive(Default)]
pub struct CollectHIRStage;

enum CallSegment {
    String(String),
    Number(usize),
}

impl CollectHIRStage {
    fn analyze_instruction(
        module: &ASTModule,
        word: &ASTWord,
        instruction: &Spanned<ASTInstruction>,
        hir_ctx: &HIRContext,
    ) -> Result<HIRInstruction, DiagnosticMessage> {
        match &instruction.value {
            ASTInstruction::Literal(literal) => match literal {
                ASTLiteral::Bool(value) => Ok(HIRInstruction::Literal(HIRLiteral::Bool(*value))),
                ASTLiteral::U8(value) => Ok(HIRInstruction::Literal(HIRLiteral::U8(*value))),
                ASTLiteral::String(value) => Ok(HIRInstruction::Literal(HIRLiteral::String(
                    value.to_owned(),
                ))),
            },
            ASTInstruction::Call(call) => {
                let segments = call
                    .0
                    .iter()
                    .map(|segment| match segment.parse::<usize>() {
                        Ok(number) => CallSegment::Number(number),
                        Err(_) => CallSegment::String(segment.clone()),
                    })
                    .collect::<Vec<_>>();

                match segments.as_slice() {
                    [name_segments @ .., CallSegment::Number(field_index)] => {
                        let Some(struct_name) = name_segments
                            .iter()
                            .map(|segment| match segment {
                                CallSegment::String(name) => Some(name.clone()),
                                CallSegment::Number(_) => None,
                            })
                            .collect::<Option<Vec<_>>>()
                            .map(DottedPath)
                        else {
                            return Err(DiagnosticMessage::SymbolNotFound {
                                name: call.to_string(),
                                span: instruction.span,
                            });
                        };
                        let resolved = (struct_name.len() == 1)
                            .then(|| module.name.extend(&struct_name))
                            .as_ref()
                            .into_iter()
                            .chain(std::iter::once(&struct_name))
                            .find_map(|candidate| {
                                hir_ctx
                                    .lookup_and_get(candidate)
                                    .map(|(id, symbol)| (id, symbol, candidate.to_string()))
                            });
                        let Some((struct_id, symbol, full_name)) = resolved else {
                            return Err(DiagnosticMessage::SymbolNotFound {
                                name: call.to_string(),
                                span: instruction.span,
                            });
                        };
                        let SymbolKind::Struct { name, fields } = symbol else {
                            return Err(DiagnosticMessage::InvalidSymbol {
                                name: full_name,
                                span: instruction.span,
                            });
                        };
                        if *field_index >= fields.len() {
                            return Err(DiagnosticMessage::InvalidFieldIndex {
                                name: name.clone(),
                                index: *field_index,
                                field_count: fields.len(),
                                span: instruction.span,
                            });
                        }
                        Ok(HIRInstruction::GetField {
                            name: name.clone(),
                            struct_id,
                            field_index: *field_index,
                        })
                    }
                    name_segments => {
                        let Some(name) = name_segments
                            .iter()
                            .map(|segment| match segment {
                                CallSegment::String(name) => Some(name.clone()),
                                CallSegment::Number(_) => None,
                            })
                            .collect::<Option<Vec<_>>>()
                            .map(DottedPath)
                        else {
                            return Err(DiagnosticMessage::SymbolNotFound {
                                name: call.to_string(),
                                span: instruction.span,
                            });
                        };
                        let Some((symbol_id, full_name)) = (name.len() == 1)
                            .then(|| module.name.extend(&name))
                            .as_ref()
                            .into_iter()
                            .chain(std::iter::once(&name))
                            .find_map(|candidate| match hir_ctx.lookup_and_get(candidate) {
                                Some((id, SymbolKind::Word { .. })) => {
                                    Some((id, candidate.to_string()))
                                }
                                _ => None,
                            })
                        else {
                            return Err(DiagnosticMessage::SymbolNotFound {
                                name: name.to_string(),
                                span: instruction.span,
                            });
                        };
                        Ok(HIRInstruction::Call {
                            name: full_name,
                            symbol_id,
                        })
                    }
                }
            }
            ASTInstruction::Pack(name) | ASTInstruction::Unpack(name) => {
                let Some(struct_id) = (name.len() == 1)
                    .then(|| module.name.extend(name))
                    .as_ref()
                    .into_iter()
                    .chain(std::iter::once(name))
                    .find_map(|name| match hir_ctx.lookup_and_get(name) {
                        Some((id, SymbolKind::Struct { .. })) => Some(id),
                        _ => None,
                    })
                else {
                    return Err(DiagnosticMessage::SymbolNotFound {
                        name: name.to_string(),
                        span: instruction.span,
                    });
                };
                let full_name = match hir_ctx.get(struct_id) {
                    Some(SymbolKind::Struct { name, .. }) => name.clone(),
                    _ => name.to_string(),
                };
                if matches!(instruction.value, ASTInstruction::Pack(_)) {
                    Ok(HIRInstruction::Pack {
                        name: full_name,
                        struct_id,
                    })
                } else {
                    Ok(HIRInstruction::Unpack {
                        name: full_name,
                        struct_id,
                    })
                }
            }
            ASTInstruction::Lambda { stack_effect, body } => {
                let mut result_stack_in = Vec::with_capacity(stack_effect.stack_in.len());
                let mut result_stack_out = Vec::with_capacity(stack_effect.stack_out.len());
                let mut result_body = Vec::with_capacity(body.len());

                let wordpath = module.name.append(&word.name.value);

                for item in &stack_effect.stack_in {
                    let ty = hir_ctx.handle_stack_item(&module.name, &wordpath, item)?;
                    result_stack_in.push(ty);
                }
                for item in &stack_effect.stack_out {
                    let ty = hir_ctx.handle_stack_item(&module.name, &wordpath, item)?;
                    result_stack_out.push(ty);
                }
                for instruction in body {
                    let instr = Self::analyze_instruction(module, word, instruction, hir_ctx)?;
                    result_body.push(Spanned::new(instr, instruction.span));
                }

                Ok(HIRInstruction::Lambda {
                    stack_in: result_stack_in,
                    stack_out: result_stack_out,
                    body: result_body,
                })
            }
        }
    }

    fn analyze_struct(
        module: &ASTModule,
        item: &ASTStruct,
        hir_ctx: &HIRContext,
    ) -> Result<HIRStruct, DiagnosticMessage> {
        let fullpath = module.name.append(item.name.as_str());
        let Some((id, SymbolKind::Struct { fields, .. })) = hir_ctx.lookup_and_get(&fullpath)
        else {
            return Err(DiagnosticMessage::SymbolNotFound {
                name: fullpath.to_string(),
                span: item.name.span,
            });
        };
        Ok(HIRStruct {
            id,
            name: Spanned::new(fullpath.to_string(), item.name.span),
            fields: fields.clone(),
        })
    }

    fn analyze_word(
        module: &ASTModule,
        is_root_module: bool,
        word: &ASTWord,
        hir_ctx: &HIRContext,
    ) -> Result<HIRWord, DiagnosticMessage> {
        let mut attributes = Vec::with_capacity(word.attributes.len());
        for attribute in &word.attributes {
            match attribute.value.as_str() {
                "builtin" => attributes.push(HIRWordAttribute::BuiltIn),
                _ => {
                    return Err(DiagnosticMessage::InvalidAttribute {
                        name: attribute.value.clone(),
                        span: attribute.span,
                    });
                }
            }
        }

        let fullpath = module.name.append(&word.name);
        let word_id =
            hir_ctx
                .lookup(&fullpath)
                .ok_or_else(|| DiagnosticMessage::SymbolNotFound {
                    name: word.name.to_string(),
                    span: word.name.span,
                })?;
        let symbol = hir_ctx
            .get(word_id)
            .ok_or_else(|| DiagnosticMessage::SymbolNotFound {
                name: word.name.to_string(),
                span: word.name.span,
            })?;
        let SymbolKind::Word {
            typevars,
            stackvars,
            stack_in,
            stack_out,
        } = symbol
        else {
            return Err(DiagnosticMessage::InvalidSymbol {
                name: word.name.to_string(),
                span: word.name.span,
            });
        };

        let mut body = Vec::with_capacity(word.body.len());
        for instruction in &word.body {
            body.push(Spanned::new(
                Self::analyze_instruction(module, word, instruction, hir_ctx)?,
                instruction.span,
            ));
        }

        Ok(HIRWord {
            id: word_id,
            signature: HIRWordSignature {
                name: Spanned::new(fullpath.to_string(), word.name.span),
                stack_effect_span: word.stack_effect.span,
                stack_in: stack_in.clone(),
                stack_out: stack_out.clone(),
                type_vars: typevars.clone(),
                stack_vars: stackvars.clone(),
            },
            body,
            attributes,
            entrypoint: is_root_module && word.name.value == "main",
            substitutions: HashMap::new(),
        })
    }

    fn analyze_module(
        module: &ASTModule,
        hir_ctx: &HIRContext,
        diagnostics: &mut Diagnostics,
    ) -> Option<HIRModule> {
        let Some(module_id) = hir_ctx.lookup(&module.name) else {
            diagnostics.emit_fatal(DiagnosticMessage::SymbolNotFound {
                name: module.name.to_string(),
                span: module.name.span,
            });
            return None;
        };

        let mut imports = vec![];
        for import in &module.imports {
            let Some(import_id) = hir_ctx.lookup(&import.value) else {
                diagnostics.emit_fatal(DiagnosticMessage::SymbolNotFound {
                    name: import.value.to_string(),
                    span: import.span,
                });
                continue;
            };
            imports.push(Spanned::new(import_id, import.span));
        }

        let mut structs = vec![];
        for item in &module.structs {
            match Self::analyze_struct(module, item, hir_ctx) {
                Ok(analyzed) => structs.push(Spanned::new(analyzed, item.span)),
                Err(error) => diagnostics.emit_fatal(error),
            }
        }

        let mut words = vec![];
        for (index, word) in module.words.iter().enumerate() {
            match Self::analyze_word(module, index == 0, word, hir_ctx) {
                Ok(analyzed) => words.push(Spanned::new(analyzed, word.name.span)),
                Err(error) => diagnostics.emit_fatal(error),
            }
        }

        Some(HIRModule {
            id: module_id,
            imports,
            structs,
            words,
        })
    }
}

impl Stage<CompileContext> for CollectHIRStage {
    type Input = (HIRContext, ASTProgram);
    type Output = (HIRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, ast): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut result = HIRProgram { modules: vec![] };

        for module in &ast.modules {
            if let Some(analyzed_module) = Self::analyze_module(module, &hir_ctx, &mut diagnostics)
            {
                result.modules.push(analyzed_module);
            }
        }

        StageResult::new(Some((hir_ctx, result)), diagnostics)
    }
}
