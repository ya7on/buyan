use std::collections::HashMap;

use crate::{
    common::{CompileContext, Spanned},
    error::CompileError,
    pipeline::Stage,
    stages::{
        parse::ast::{ASTInstruction, ASTLiteral, ASTModule, ASTProgram, ASTWord},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::{
                HIRInstruction, HIRLiteral, HIRModule, HIRProgram, HIRWord, HIRWordAttribute,
                HIRWordSignature,
            },
        },
    },
};

#[derive(Default)]
pub struct CollectHIRStage;

impl CollectHIRStage {
    fn analyze_instruction(
        module: &ASTModule,
        word: &ASTWord,
        instruction: &Spanned<ASTInstruction>,
        hir_ctx: &HIRContext,
    ) -> Result<HIRInstruction, Vec<CompileError>> {
        match &instruction.value {
            ASTInstruction::Literal(literal) => match literal {
                ASTLiteral::U8(value) => Ok(HIRInstruction::Literal(HIRLiteral::U8(*value))),
                ASTLiteral::String(value) => Ok(HIRInstruction::Literal(HIRLiteral::String(
                    value.to_owned(),
                ))),
            },
            ASTInstruction::Call(call) => {
                let full_name = if call.len() == 1 {
                    module.name.extend(call)
                } else {
                    call.clone()
                };
                let Some((symbol_id, SymbolKind::Word { .. })) = hir_ctx.lookup_and_get(&full_name)
                else {
                    return Err(vec![CompileError::SymbolNotFound {
                        name: full_name.to_string(),
                        span: instruction.span,
                    }]);
                };
                Ok(HIRInstruction::Call {
                    name: full_name.to_string(),
                    symbol_id,
                })
            }
            ASTInstruction::Lambda { stack_effect, body } => {
                let mut result_stack_in = Vec::with_capacity(stack_effect.stack_in.len());
                let mut result_stack_out = Vec::with_capacity(stack_effect.stack_out.len());
                let mut result_body = Vec::with_capacity(body.len());

                let wordpath = module.name.append(&word.name.value);

                for item in &stack_effect.stack_in {
                    let ty = hir_ctx
                        .handle_stack_item(&wordpath, item)
                        .map_err(|err| vec![err])?;
                    result_stack_in.push(ty);
                }
                for item in &stack_effect.stack_out {
                    let ty = hir_ctx
                        .handle_stack_item(&wordpath, item)
                        .map_err(|err| vec![err])?;
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

    fn analyze_word(
        module: &ASTModule,
        is_root_module: bool,
        word: &ASTWord,
        hir_ctx: &HIRContext,
    ) -> Result<HIRWord, Vec<CompileError>> {
        let mut attributes = Vec::with_capacity(word.attributes.len());
        for attribute in &word.attributes {
            match attribute.value.as_str() {
                "builtin" => attributes.push(HIRWordAttribute::BuiltIn),
                _ => {
                    return Err(vec![CompileError::InvalidAttribute {
                        name: attribute.value.clone(),
                        span: attribute.span,
                    }]);
                }
            }
        }

        let fullpath = module.name.append(&word.name);
        let word_id = hir_ctx.lookup(&fullpath).ok_or_else(|| {
            vec![CompileError::SymbolNotFound {
                name: word.name.to_string(),
                span: word.name.span,
            }]
        })?;
        let symbol = hir_ctx.get(word_id).ok_or_else(|| {
            vec![CompileError::SymbolNotFound {
                name: word.name.to_string(),
                span: word.name.span,
            }]
        })?;
        let SymbolKind::Word {
            typevars,
            stackvars,
            stack_in,
            stack_out,
        } = symbol
        else {
            return Err(vec![CompileError::InvalidSymbol {
                name: word.name.to_string(),
                span: word.name.span,
            }]);
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
    ) -> Result<HIRModule, Vec<CompileError>> {
        let module_id = hir_ctx.lookup(&module.name).ok_or_else(|| {
            vec![CompileError::SymbolNotFound {
                name: module.name.to_string(),
                span: module.name.span,
            }]
        })?;

        let mut imports = vec![];
        for import in &module.imports {
            let import_id = hir_ctx.lookup(&import.value).ok_or_else(|| {
                vec![CompileError::SymbolNotFound {
                    name: import.value.to_string(),
                    span: import.span,
                }]
            })?;
            imports.push(Spanned::new(import_id, import.span));
        }

        let mut words = vec![];
        for (index, word) in module.words.iter().enumerate() {
            words.push(Spanned::new(
                Self::analyze_word(module, index == 0, word, hir_ctx)?,
                word.name.span,
            ));
        }

        Ok(HIRModule {
            id: module_id,
            imports,
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
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut result = HIRProgram { modules: vec![] };
        let mut errors = vec![];

        for module in &ast.modules {
            match Self::analyze_module(module, &hir_ctx) {
                Ok(analyzed_module) => {
                    result.modules.push(analyzed_module);
                }
                Err(module_errors) => {
                    errors.extend(module_errors);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((hir_ctx, result))
    }
}
