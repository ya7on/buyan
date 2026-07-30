use crate::{
    common::{CompileContext, Span, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        lower::{
            context::{IRContext, WordId},
            ir::{
                BasicBlockId, IRBasicBlock, IRConstant, IRInstruction, IRProgram, IRTerminator,
                IRType, IRWord,
            },
            monomorphize::substitute_type,
        },
        semantic::{
            context::SymbolId,
            hir::{HIRConst, HIRInstruction, HIRLiteral, HIRProgram, HIRType},
        },
    },
};

#[derive(Debug, Default)]
pub struct BasicBlockBuilder {
    pub instructions: Vec<Spanned<IRInstruction>>,
}

impl BasicBlockBuilder {
    pub fn push(&mut self, instruction: Spanned<IRInstruction>) {
        self.instructions.push(instruction);
    }

    #[must_use]
    pub fn build(self, terminator: Spanned<IRTerminator>) -> IRBasicBlock {
        IRBasicBlock {
            instructions: self.instructions,
            terminator,
        }
    }
}

#[derive(Debug, Default)]
pub struct LowerStage;

impl LowerStage {
    fn lower_type(ir_ctx: &IRContext, ty: &HIRType) -> Option<IRType> {
        match ty {
            HIRType::BuiltIn(symbol_id) | HIRType::Struct(symbol_id) => ir_ctx
                .symbol_id_to_type_id
                .get(symbol_id)
                .and_then(|type_id| ir_ctx.get_type(*type_id))
                .cloned(),
            HIRType::StackVar(_) | HIRType::TypeVar(_) => None,
            HIRType::Lambda { .. } => Some(IRType::Lambda),
            HIRType::Array { element_type, size } => {
                let HIRConst::Value(size) = size else {
                    return None;
                };
                Some(IRType::Array {
                    element_type: Box::new(Self::lower_type(ir_ctx, element_type)?),
                    size: *size,
                })
            }
        }
    }

    fn lower_ir_word(
        ir_ctx: &IRContext,
        body: &[Spanned<HIRInstruction>],
        entrypoint: bool,
        span: Span,
        substitutions: &[(SymbolId, HIRType)],
        lambda_words: &mut Vec<Spanned<IRWord>>,
        base_word_count: usize,
    ) -> Result<IRWord, Vec<DiagnosticMessage>> {
        let mut errors = Vec::new();
        let mut blocks = Vec::new();
        let mut basicblock = BasicBlockBuilder::default();
        'instructions: for instruction in body {
            match &instruction.value {
                HIRInstruction::Call {
                    name,
                    symbol_id,
                    type_args,
                } => {
                    let Some(concrete_type_args) = type_args
                        .iter()
                        .map(|ty| substitute_type(ty, substitutions))
                        .collect::<Option<Vec<_>>>()
                    else {
                        errors.push(DiagnosticMessage::CannotInferType {
                            label: format!("cannot resolve type arguments for '{name}'"),
                            span: instruction.span,
                        });
                        continue;
                    };
                    let Some(type_args) = concrete_type_args
                        .iter()
                        .map(|ty| Self::lower_type(ir_ctx, ty))
                        .collect::<Option<Vec<_>>>()
                    else {
                        errors.push(DiagnosticMessage::CannotInferType {
                            label: format!("cannot lower type arguments for '{name}'"),
                            span: instruction.span,
                        });
                        continue;
                    };

                    match name.as_str() {
                        // Builtin call
                        "std.cfg.if" => {
                            let current_block_id = blocks.len();
                            let then_branch = BasicBlockId(current_block_id + 1);
                            let else_branch = BasicBlockId(current_block_id + 2);
                            let join_branch = BasicBlockId(current_block_id + 3);

                            blocks.push(basicblock.build(Spanned::new(
                                IRTerminator::BranchIfZero {
                                    then_branch,
                                    else_branch,
                                },
                                instruction.span,
                            )));
                            blocks.push(
                                BasicBlockBuilder {
                                    instructions: vec![Spanned::new(
                                        IRInstruction::CallIndirect,
                                        instruction.span,
                                    )],
                                }
                                .build(Spanned::new(
                                    IRTerminator::Branch {
                                        branch: join_branch,
                                    },
                                    instruction.span,
                                )),
                            );
                            blocks.push(
                                BasicBlockBuilder {
                                    instructions: vec![Spanned::new(
                                        IRInstruction::CallIndirect,
                                        instruction.span,
                                    )],
                                }
                                .build(Spanned::new(
                                    IRTerminator::Branch {
                                        branch: join_branch,
                                    },
                                    instruction.span,
                                )),
                            );

                            basicblock = BasicBlockBuilder::default();
                        }
                        "std.io.print" => {
                            basicblock.push(Spanned::new(IRInstruction::Print, instruction.span));
                        }
                        "std.io.input" => {
                            basicblock.push(Spanned::new(IRInstruction::Input, instruction.span));
                        }
                        "std.io.flush" => {
                            basicblock.push(Spanned::new(IRInstruction::Flush, instruction.span));
                        }
                        "std.stack.call" => {
                            basicblock
                                .push(Spanned::new(IRInstruction::CallIndirect, instruction.span));
                        }
                        "std.stack.drop" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Drop { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.stack.dup" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Dup { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.stack.swap" => {
                            let (Some(lower), Some(upper)) = (type_args.first(), type_args.get(1))
                            else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type arguments for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Swap {
                                    lower: lower.clone(),
                                    upper: upper.clone(),
                                },
                                instruction.span,
                            ));
                        }
                        "std.math.add" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Add { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.sub" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Sub { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.mul" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Mul { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.div" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Div { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.eq" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Eq { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.gt" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Gt { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.math.lt" => {
                            let Some(ty) = type_args.first() else {
                                errors.push(DiagnosticMessage::CannotInferType {
                                    label: format!("missing type argument for '{name}'"),
                                    span: instruction.span,
                                });
                                continue;
                            };
                            basicblock.push(Spanned::new(
                                IRInstruction::Lt { ty: ty.clone() },
                                instruction.span,
                            ));
                        }
                        "std.array.index" => {
                            basicblock
                                .push(Spanned::new(IRInstruction::ArrayIndex, instruction.span));
                        }
                        // Real word call
                        _ => match ir_ctx.get_word_id(*symbol_id, &concrete_type_args) {
                            Some(word_id) => basicblock.push(Spanned::new(
                                IRInstruction::CallDirect { word_id },
                                instruction.span,
                            )),
                            None => errors.push(DiagnosticMessage::SymbolNotFound {
                                name: name.clone(),
                                span: instruction.span,
                            }),
                        },
                    }
                }
                HIRInstruction::Literal(literal) => match literal {
                    HIRLiteral::Bool(value) => {
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant {
                                value: IRConstant::Bool(*value),
                            },
                            instruction.span,
                        ));
                    }
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
                                value: IRConstant::String(value.clone()),
                            },
                            instruction.span,
                        ));
                    }
                },
                HIRInstruction::Pack { name, struct_id } => {
                    let Some(type_id) = ir_ctx.symbol_id_to_type_id.get(struct_id).copied() else {
                        errors.push(DiagnosticMessage::SymbolNotFound {
                            name: name.clone(),
                            span: instruction.span,
                        });
                        continue;
                    };
                    let Some(IRType::Struct { fields }) = ir_ctx.get_type(type_id) else {
                        errors.push(DiagnosticMessage::SymbolNotFound {
                            name: name.clone(),
                            span: instruction.span,
                        });
                        continue;
                    };
                    basicblock.push(Spanned::new(
                        IRInstruction::Pack {
                            type_id,
                            field_count: fields.len(),
                        },
                        instruction.span,
                    ));
                }
                HIRInstruction::Unpack { name, struct_id } => {
                    let Some(type_id) = ir_ctx.symbol_id_to_type_id.get(struct_id).copied() else {
                        errors.push(DiagnosticMessage::SymbolNotFound {
                            name: name.clone(),
                            span: instruction.span,
                        });
                        continue;
                    };
                    basicblock.push(Spanned::new(
                        IRInstruction::Unpack { type_id },
                        instruction.span,
                    ));
                }
                HIRInstruction::GetField {
                    name,
                    struct_id,
                    field_index,
                } => {
                    let Some(type_id) = ir_ctx.symbol_id_to_type_id.get(struct_id).copied() else {
                        errors.push(DiagnosticMessage::SymbolNotFound {
                            name: name.clone(),
                            span: instruction.span,
                        });
                        continue;
                    };
                    basicblock.push(Spanned::new(
                        IRInstruction::GetField {
                            type_id,
                            field_index: *field_index,
                        },
                        instruction.span,
                    ));
                }
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
                        substitutions,
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
                HIRInstruction::Array { elements } => {
                    for element in elements {
                        let HIRInstruction::Literal(literal) = &element.value else {
                            errors.push(DiagnosticMessage::InvalidArrayValue {
                                label: "array elements must be literals".to_string(),
                                span: element.span,
                            });
                            continue 'instructions;
                        };
                        let value = match literal {
                            HIRLiteral::Bool(value) => IRConstant::Bool(*value),
                            HIRLiteral::U8(value) => IRConstant::U8(*value),
                            HIRLiteral::String(value) => IRConstant::String(value.clone()),
                        };
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant { value },
                            element.span,
                        ));
                    }
                    basicblock.push(Spanned::new(
                        IRInstruction::PackArray {
                            element_count: elements.len(),
                        },
                        instruction.span,
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        blocks.push(basicblock.build(Spanned::new(IRTerminator::End, span)));

        Ok(IRWord { entrypoint, blocks })
    }
}

impl Stage<CompileContext> for LowerStage {
    type Input = (IRContext, HIRProgram);
    type Output = (IRContext, IRProgram);

    fn execute(
        &mut self,
        (ir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut result = IRProgram { words: Vec::new() };
        let mut lambda_words = Vec::new();
        let base_word_count = ir_ctx.words.len();

        let words = hir_program
            .modules
            .iter()
            .flat_map(|module| module.words.iter())
            .map(|word| (word.id, word))
            .collect::<std::collections::HashMap<_, _>>();

        for instance in &ir_ctx.words {
            let Some(word) = words.get(&instance.source_word) else {
                diagnostics.emit_fatal(DiagnosticMessage::Unknown {
                    label: format!("source word '{}' not found", instance.source_word.0),
                });
                continue;
            };
            let substitutions = word
                .signature
                .type_vars
                .iter()
                .map(|var| var.value)
                .zip(instance.type_args.iter().cloned())
                .collect::<Vec<_>>();
            match Self::lower_ir_word(
                &ir_ctx,
                &word.body,
                word.entrypoint,
                word.span,
                &substitutions,
                &mut lambda_words,
                base_word_count,
            ) {
                Ok(ir_word) => result.words.push(Spanned::new(ir_word, word.span)),
                Err(err) => {
                    for error in err {
                        diagnostics.emit_fatal(error);
                    }
                }
            }
        }
        result.words.extend(lambda_words);

        StageResult::new(Some((ir_ctx, result)), diagnostics)
    }
}
