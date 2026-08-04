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
            hir::{HIRInstruction, HIRLiteral, HIRProgram, HIRType},
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
        }
    }

    fn lower_ir_word(
        ir_ctx: &IRContext,
        word_id: WordId,
        name: String,
        body: &[Spanned<HIRInstruction>],
        entrypoint: bool,
        span: Span,
        substitutions: &[(SymbolId, HIRType)],
        lambda_words: &mut Vec<Spanned<IRWord>>,
        base_word_count: usize,
        static_data: &mut Vec<u8>,
    ) -> Result<IRWord, Vec<DiagnosticMessage>> {
        let mut errors = Vec::new();
        let mut blocks = Vec::new();
        let mut basicblock = BasicBlockBuilder::default();
        for instruction in body {
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
                            let Some(else_lambda) = basicblock.instructions.pop() else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.if' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let IRInstruction::PushLambda {
                                word_id: else_word_id,
                            } = else_lambda.value
                            else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.if' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let Some(then_lambda) = basicblock.instructions.pop() else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.if' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let IRInstruction::PushLambda {
                                word_id: then_word_id,
                            } = then_lambda.value
                            else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.if' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let current_block_id = blocks.len();
                            let decision_branch = BasicBlockId(current_block_id + 1);
                            let then_branch = BasicBlockId(current_block_id + 2);
                            let else_branch = BasicBlockId(current_block_id + 3);
                            let join_branch = BasicBlockId(current_block_id + 4);

                            blocks.push(basicblock.build(Spanned::new(
                                IRTerminator::Branch {
                                    branch: decision_branch,
                                },
                                instruction.span,
                            )));
                            blocks.push(BasicBlockBuilder::default().build(Spanned::new(
                                IRTerminator::BranchIfZero {
                                    then_branch,
                                    else_branch,
                                },
                                instruction.span,
                            )));
                            blocks.push(
                                BasicBlockBuilder {
                                    instructions: vec![Spanned::new(
                                        IRInstruction::CallDirect {
                                            word_id: then_word_id,
                                        },
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
                                        IRInstruction::CallDirect {
                                            word_id: else_word_id,
                                        },
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
                        "std.cfg.while" => {
                            let Some(body_lambda) = basicblock.instructions.pop() else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.while' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let IRInstruction::PushLambda {
                                word_id: body_word_id,
                            } = body_lambda.value
                            else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.while' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let Some(condition_lambda) = basicblock.instructions.pop() else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.while' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let IRInstruction::PushLambda {
                                word_id: condition_word_id,
                            } = condition_lambda.value
                            else {
                                errors.push(DiagnosticMessage::InvalidStack {
                                    label: "'std.cfg.while' expects two immediately preceding lambda literals".to_string(),
                                    span: instruction.span,
                                    additional_spans: Vec::new(),
                                    expected_stack: Vec::new(),
                                    actual_stack: Vec::new(),
                                });
                                continue;
                            };
                            let current_block_id = blocks.len();
                            let condition_branch = BasicBlockId(current_block_id + 1);
                            let test_branch = BasicBlockId(current_block_id + 2);
                            let body_branch = BasicBlockId(current_block_id + 3);
                            let exit_branch = BasicBlockId(current_block_id + 4);

                            blocks.push(basicblock.build(Spanned::new(
                                IRTerminator::Branch {
                                    branch: condition_branch,
                                },
                                instruction.span,
                            )));
                            blocks.push(
                                BasicBlockBuilder {
                                    instructions: vec![Spanned::new(
                                        IRInstruction::CallDirect {
                                            word_id: condition_word_id,
                                        },
                                        instruction.span,
                                    )],
                                }
                                .build(Spanned::new(
                                    IRTerminator::Branch {
                                        branch: test_branch,
                                    },
                                    instruction.span,
                                )),
                            );
                            blocks.push(BasicBlockBuilder::default().build(Spanned::new(
                                IRTerminator::BranchIfZero {
                                    then_branch: body_branch,
                                    else_branch: exit_branch,
                                },
                                instruction.span,
                            )));
                            blocks.push(
                                BasicBlockBuilder {
                                    instructions: vec![Spanned::new(
                                        IRInstruction::CallDirect {
                                            word_id: body_word_id,
                                        },
                                        instruction.span,
                                    )],
                                }
                                .build(Spanned::new(
                                    IRTerminator::Branch {
                                        branch: condition_branch,
                                    },
                                    instruction.span,
                                )),
                            );
                            blocks.push(BasicBlockBuilder::default().build(Spanned::new(
                                IRTerminator::Branch {
                                    branch: BasicBlockId(current_block_id + 5),
                                },
                                instruction.span,
                            )));

                            basicblock = BasicBlockBuilder::default();
                        }
                        "std.io.print" => {
                            basicblock.push(Spanned::new(IRInstruction::Print, instruction.span));
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
                        "std.u8.add" => basicblock.push(Spanned::new(
                            IRInstruction::Add { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.sub" => basicblock.push(Spanned::new(
                            IRInstruction::Sub { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.mul" => basicblock.push(Spanned::new(
                            IRInstruction::Mul { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.div" => basicblock.push(Spanned::new(
                            IRInstruction::Div { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.eq" => basicblock.push(Spanned::new(
                            IRInstruction::Eq { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.gt" => basicblock.push(Spanned::new(
                            IRInstruction::Gt { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u8.lt" => basicblock.push(Spanned::new(
                            IRInstruction::Lt { ty: IRType::U8 },
                            instruction.span,
                        )),
                        "std.u16.add" => basicblock.push(Spanned::new(
                            IRInstruction::Add { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.sub" => basicblock.push(Spanned::new(
                            IRInstruction::Sub { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.mul" => basicblock.push(Spanned::new(
                            IRInstruction::Mul { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.div" => basicblock.push(Spanned::new(
                            IRInstruction::Div { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.eq" => basicblock.push(Spanned::new(
                            IRInstruction::Eq { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.gt" => basicblock.push(Spanned::new(
                            IRInstruction::Gt { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.u16.lt" => basicblock.push(Spanned::new(
                            IRInstruction::Lt { ty: IRType::U16 },
                            instruction.span,
                        )),
                        "std.bool.eq" => basicblock.push(Spanned::new(
                            IRInstruction::Eq { ty: IRType::Bool },
                            instruction.span,
                        )),
                        "std.ptr.offset" => {
                            basicblock.push(Spanned::new(
                                IRInstruction::Add { ty: IRType::U16 },
                                instruction.span,
                            ));
                        }
                        "std.unsafe.mem.load" => {
                            basicblock.push(Spanned::new(IRInstruction::Load, instruction.span));
                        }
                        "std.unsafe.mem.store" => {
                            basicblock.push(Spanned::new(IRInstruction::Store, instruction.span));
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
                    HIRLiteral::U16(value) => {
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant {
                                value: IRConstant::U16(*value),
                            },
                            instruction.span,
                        ));
                    }
                    HIRLiteral::String { value, struct_id } => {
                        let Some(address) = u16::try_from(static_data.len()).ok() else {
                            errors.push(DiagnosticMessage::DataIsTooBig {
                                span: instruction.span,
                            });
                            continue;
                        };
                        let bytes = value.as_bytes();
                        if bytes.len() > usize::from(u8::MAX)
                            || static_data.len() + bytes.len() + 1 > usize::from(u16::MAX) + 1
                        {
                            errors.push(DiagnosticMessage::DataIsTooBig {
                                span: instruction.span,
                            });
                            continue;
                        }
                        static_data.push(bytes.len() as u8);
                        static_data.extend_from_slice(bytes);
                        let Some(type_id) = ir_ctx.symbol_id_to_type_id.get(struct_id).copied()
                        else {
                            errors.push(DiagnosticMessage::SymbolNotFound {
                                name: "std.str.Str".to_string(),
                                span: instruction.span,
                            });
                            continue;
                        };
                        basicblock.push(Spanned::new(
                            IRInstruction::PushConstant {
                                value: IRConstant::U16(address),
                            },
                            instruction.span,
                        ));
                        basicblock.push(Spanned::new(
                            IRInstruction::Pack { type_id },
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
                    basicblock.push(Spanned::new(
                        IRInstruction::Pack { type_id },
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
                            word_id,
                            name: format!("lambda_{}", word_id.id()),
                            entrypoint: false,
                            blocks: Vec::new(),
                        },
                        instruction.span,
                    ));

                    let lambda_word = match Self::lower_ir_word(
                        ir_ctx,
                        word_id,
                        format!("lambda_{}", word_id.id()),
                        body,
                        false,
                        instruction.span,
                        substitutions,
                        lambda_words,
                        base_word_count,
                        static_data,
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

        blocks.push(basicblock.build(Spanned::new(IRTerminator::End, span)));

        Ok(IRWord {
            word_id,
            name,
            entrypoint,
            blocks,
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
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut result = IRProgram {
            words: Vec::new(),
            types: ir_ctx.types.clone(),
            static_data: Vec::new(),
        };
        let mut lambda_words = Vec::new();
        let base_word_count = ir_ctx.words.len();

        let words = hir_program
            .modules
            .iter()
            .flat_map(|module| module.words.iter())
            .map(|word| (word.id, word))
            .collect::<std::collections::HashMap<_, _>>();

        for (index, instance) in ir_ctx.words.iter().enumerate() {
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
                WordId(index),
                instance.name.clone(),
                &word.body,
                word.entrypoint,
                word.span,
                &substitutions,
                &mut lambda_words,
                base_word_count,
                &mut result.static_data,
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
