use crate::{
    common::CompileContext,
    error::CompileError,
    pipeline::Stage,
    stages::{parse::ast::ASTProgram, semantic::context::HIRContext},
};

#[derive(Default)]
pub struct CollectNamesStage;

impl Stage<CompileContext> for CollectNamesStage {
    type Input = ASTProgram;
    type Output = (HIRContext, ASTProgram);

    fn execute(
        &mut self,
        input: Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut errors = Vec::new();

        let mut context = HIRContext::default();

        for module in &input.modules {
            let module_id = match context.register_module(module) {
                Ok(module_id) => module_id,
                Err(err) => {
                    errors.push(err);
                    continue;
                }
            };

            for word in &module.words {
                match context.register_word(module_id, word) {
                    Ok(_) => {}
                    Err(err) => {
                        errors.push(err);
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok((context, input))
    }
}
