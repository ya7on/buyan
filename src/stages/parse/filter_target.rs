use crate::{
    common::{CompileContext, CompileTarget},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::parse::ast::{ASTProgram, ASTWord},
};

#[derive(Debug, Default)]
pub struct FilterTargetStage;

impl FilterTargetStage {
    fn filter_word(
        mut word: ASTWord,
        target: CompileTarget,
        diagnostics: &mut Diagnostics,
    ) -> Option<ASTWord> {
        let targets = word
            .attributes
            .iter()
            .filter(|attribute| attribute.value.name == "target")
            .map(|attribute| {
                let Some(value) = attribute.value.value.as_deref() else {
                    return Err(DiagnosticMessage::InvalidAttribute {
                        name: "target".to_string(),
                        reason: "expected #[target = \"<target>\"]".to_string(),
                        span: attribute.span,
                    });
                };
                CompileTarget::try_from(value).map_err(|()| DiagnosticMessage::InvalidAttribute {
                    name: "target".to_string(),
                    reason: format!("unknown target \"{value}\";"),
                    span: attribute.span,
                })
            })
            .collect::<Result<Vec<_>, _>>();
        let targets = match targets {
            Ok(targets) => targets,
            Err(diagnostic) => {
                diagnostics.emit_fatal(diagnostic);
                return None;
            }
        };

        if !targets.is_empty() && !targets.contains(&target) {
            return None;
        }

        word.attributes
            .retain(|attribute| attribute.value.name != "target");
        Some(word)
    }
}

impl Stage<CompileContext> for FilterTargetStage {
    type Input = ASTProgram;
    type Output = ASTProgram;

    fn execute(
        &mut self,
        mut input: Self::Input,
        context: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();

        for module in &mut input.modules {
            module.words = std::mem::take(&mut module.words)
                .into_iter()
                .filter_map(|word| {
                    Self::filter_word(word.value, context.target, &mut diagnostics)
                        .map(|value| crate::common::Spanned::new(value, word.span))
                })
                .collect();
        }

        StageResult::new(Some(input), diagnostics)
    }
}
