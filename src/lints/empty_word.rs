use crate::{
    error::{DiagnosticMessage, Diagnostics},
    lints::Lint,
    stages::semantic::{
        context::HIRContext,
        hir::{HIRWord, HIRWordAttribute},
    },
};

pub struct EmptyWord;

impl Lint for EmptyWord {
    fn check_word(&mut self, _: &HIRContext, word: &HIRWord, diagnostics: &mut Diagnostics) {
        if word.body.is_empty() && !word.attributes.contains(&HIRWordAttribute::BuiltIn) {
            diagnostics.emit_warning(DiagnosticMessage::EmptyWord {
                span: word.signature.name.span,
            });
        }
    }
}
