use crate::{
    common::CompileContext,
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        lower::context::{IRContext, TypeIRInfo, WordIRInfo},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::HIRProgram,
        },
    },
};

#[derive(Debug, Default)]
pub struct CollectSymbolsStage;

impl Stage<CompileContext> for CollectSymbolsStage {
    type Input = (HIRContext, HIRProgram);
    type Output = (IRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut ir_ctx = IRContext::default();

        for module in &hir_program.modules {
            for item in &module.structs {
                let Some(SymbolKind::Struct { fields, .. }) = hir_ctx.get(item.id) else {
                    diagnostics.emit_fatal(DiagnosticMessage::SymbolNotFound {
                        name: item.name.to_string(),
                        span: item.span,
                    });
                    continue;
                };
                ir_ctx.register_type(
                    item.id,
                    TypeIRInfo {
                        name: item.name.to_string(),
                        field_count: fields.len(),
                    },
                );
            }
        }

        for module in &hir_program.modules {
            for word in &module.words {
                let Some(SymbolKind::Word { .. }) = hir_ctx.get(word.id) else {
                    diagnostics.emit_fatal(DiagnosticMessage::SymbolNotFound {
                        name: word.signature.name.to_string(),
                        span: word.span,
                    });
                    continue;
                };
                ir_ctx.register_word(
                    word.id,
                    WordIRInfo {
                        name: word.signature.name.to_string(),
                    },
                );
            }
        }

        StageResult::new(Some((ir_ctx, hir_program)), diagnostics)
    }
}
