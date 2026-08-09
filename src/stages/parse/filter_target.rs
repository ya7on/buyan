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
        let target_attributes = word
            .attributes
            .iter()
            .filter(|attribute| attribute.value.name == "target")
            .collect::<Vec<_>>();

        let Some(attribute) = target_attributes.first() else {
            return Some(word);
        };
        if target_attributes.len() > 1 {
            diagnostics.emit_fatal(DiagnosticMessage::InvalidAttribute {
                name: "target".to_string(),
                reason: "only one target attribute is allowed per word".to_string(),
                span: attribute.span,
            });
            return None;
        }

        let Some(value) = attribute.value.value.as_deref() else {
            diagnostics.emit_fatal(DiagnosticMessage::InvalidAttribute {
                name: "target".to_string(),
                reason: "expected #[target = \"<target>\"]".to_string(),
                span: attribute.span,
            });
            return None;
        };
        let Ok(attribute_target) = CompileTarget::try_from(value) else {
            diagnostics.emit_fatal(DiagnosticMessage::InvalidAttribute {
                name: "target".to_string(),
                reason: format!("unknown target \"{value}\";"),
                span: attribute.span,
            });
            return None;
        };

        if attribute_target != target {
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
