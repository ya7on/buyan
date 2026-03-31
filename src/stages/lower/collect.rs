use crate::{
    common::CompileContext,
    error::CompileError,
    pipeline::Stage,
    stages::{
        lower::context::{IRContext, WordIRInfo},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::HIRProgram,
        },
    },
};

#[derive(Default)]
pub struct CollectSymbolsStage;

impl Stage<CompileContext> for CollectSymbolsStage {
    type Input = (HIRContext, HIRProgram);
    type Output = (IRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut errors = Vec::new();
        let mut ir_ctx = IRContext::default();

        for module in &hir_program.modules {
            for word in &module.words {
                let Some(SymbolKind::Word { .. }) = hir_ctx.get(word.id) else {
                    errors.push(CompileError::SymbolNotFound {
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

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((ir_ctx, hir_program))
    }
}
