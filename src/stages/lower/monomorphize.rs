use std::collections::{HashMap, VecDeque};

use crate::{
    common::{CompileContext, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        lower::context::{IRContext, WordIRInfo, WordId},
        semantic::{
            context::SymbolId,
            hir::{HIRInstruction, HIRProgram, HIRType, HIRWord, HIRWordAttribute},
        },
    },
};

pub(crate) fn substitute_type(
    ty: &HIRType,
    substitutions: &[(SymbolId, HIRType)],
) -> Option<HIRType> {
    match ty {
        HIRType::BuiltIn(_) | HIRType::Struct(_) => Some(ty.clone()),
        HIRType::TypeVar(symbol_id) => substitutions
            .iter()
            .find(|(candidate, _)| candidate == symbol_id)
            .map(|(_, ty)| ty.clone()),
        HIRType::StackVar(_) => None,
        HIRType::Lambda {
            stack_in,
            stack_out,
        } => Some(HIRType::Lambda {
            stack_in: stack_in
                .iter()
                .map(|ty| substitute_type(ty, substitutions))
                .collect::<Option<Vec<_>>>()?,
            stack_out: stack_out
                .iter()
                .map(|ty| substitute_type(ty, substitutions))
                .collect::<Option<Vec<_>>>()?,
        }),
        HIRType::Array { element_type, size } => Some(HIRType::Array {
            element_type: Box::new(substitute_type(element_type, substitutions)?),
            size: size.clone(),
        }),
    }
}

fn collect_calls(
    body: &[Spanned<HIRInstruction>],
    substitutions: &[(SymbolId, HIRType)],
    words: &HashMap<SymbolId, &Spanned<HIRWord>>,
    ir_ctx: &mut IRContext,
    queue: &mut VecDeque<WordId>,
    diagnostics: &mut Diagnostics,
) {
    for instruction in body {
        match &instruction.value {
            HIRInstruction::Call {
                name,
                symbol_id,
                type_args,
            } => {
                let Some(word) = words.get(symbol_id) else {
                    diagnostics.emit_fatal(DiagnosticMessage::SymbolNotFound {
                        name: name.clone(),
                        span: instruction.span,
                    });
                    continue;
                };
                if word.attributes.contains(&HIRWordAttribute::BuiltIn) {
                    continue;
                }
                let Some(type_args) = type_args
                    .iter()
                    .map(|ty| substitute_type(ty, substitutions))
                    .collect::<Option<Vec<_>>>()
                else {
                    diagnostics.emit_fatal(DiagnosticMessage::CannotInferType {
                        label: format!("cannot resolve type arguments for '{name}'"),
                        span: instruction.span,
                    });
                    continue;
                };
                if let Some(word_id) = ir_ctx.register_word(WordIRInfo {
                    name: word.signature.name.to_string(),
                    source_word: *symbol_id,
                    type_args,
                }) {
                    queue.push_back(word_id);
                }
            }
            HIRInstruction::Lambda { body, .. } => {
                collect_calls(body, substitutions, words, ir_ctx, queue, diagnostics);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct MonomorphizeStage;

impl Stage<CompileContext> for MonomorphizeStage {
    type Input = (IRContext, HIRProgram);
    type Output = (IRContext, HIRProgram);

    fn execute(
        &mut self,
        (mut ir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut words = HashMap::new();
        let mut queue = VecDeque::new();

        for module in &hir_program.modules {
            for word in &module.words {
                words.insert(word.id, word);
                if word.entrypoint
                    && !word.attributes.contains(&HIRWordAttribute::BuiltIn)
                    && let Some(word_id) = ir_ctx.register_word(WordIRInfo {
                        name: word.signature.name.to_string(),
                        source_word: word.id,
                        type_args: Vec::new(),
                    })
                {
                    queue.push_back(word_id);
                }
            }
        }

        while let Some(word_id) = queue.pop_front() {
            let instance = ir_ctx.words[word_id.id()].clone();
            let Some(word) = words.get(&instance.source_word) else {
                diagnostics.emit_fatal(DiagnosticMessage::Unknown {
                    label: format!("source word '{}' not found", instance.source_word.0),
                });
                continue;
            };
            if word.signature.type_vars.len() != instance.type_args.len() {
                diagnostics.emit_fatal(DiagnosticMessage::CannotInferType {
                    label: format!(
                        "cannot resolve type arguments for '{}'",
                        word.signature.name.value
                    ),
                    span: word.span,
                });
                continue;
            }
            let substitutions = word
                .signature
                .type_vars
                .iter()
                .map(|var| var.value)
                .zip(instance.type_args)
                .collect::<Vec<_>>();
            collect_calls(
                &word.body,
                &substitutions,
                &words,
                &mut ir_ctx,
                &mut queue,
                &mut diagnostics,
            );
        }

        StageResult::new(Some((ir_ctx, hir_program)), diagnostics)
    }
}
