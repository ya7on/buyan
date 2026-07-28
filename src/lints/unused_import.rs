use crate::{
    common::{Span, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    lints::Lint,
    stages::semantic::{
        context::{HIRContext, SymbolKind},
        hir::{HIRInstruction, HIRModule, HIRProgram},
    },
};

#[derive(Debug)]
struct Import {
    name: String,
    span: Span,
}

#[derive(Debug, Default)]
pub struct UnusedImport {
    imports: Vec<Vec<Import>>,
}

impl Lint for UnusedImport {
    fn check_module(
        &mut self,
        ctx: &HIRContext,
        module: &HIRModule,
        _diagnostics: &mut Diagnostics,
    ) {
        self.imports.push(
            module
                .imports
                .iter()
                .filter_map(|import| {
                    let SymbolKind::Module { name } = ctx.get(import.value)? else {
                        return None;
                    };
                    Some(Import {
                        name: name.to_string(),
                        span: import.span,
                    })
                })
                .collect(),
        );
    }

    fn check_instruction(
        &mut self,
        _ctx: &HIRContext,
        instruction: &Spanned<HIRInstruction>,
        _diagnostics: &mut Diagnostics,
    ) {
        let name = match &instruction.value {
            HIRInstruction::Call { name, .. }
            | HIRInstruction::Pack { name, .. }
            | HIRInstruction::Unpack { name, .. }
            | HIRInstruction::GetField { name, .. } => name,
            HIRInstruction::Literal(_)
            | HIRInstruction::Lambda { .. }
            | HIRInstruction::Array { .. } => return,
        };
        let Some(imports) = self.imports.last_mut() else {
            return;
        };
        imports.retain(|import| {
            name != &import.name
                && !name
                    .strip_prefix(&import.name)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        });
    }

    fn finish(&mut self, _ctx: &HIRContext, _program: &HIRProgram, diagnostics: &mut Diagnostics) {
        for import in self.imports.iter().flatten() {
            diagnostics.emit_warning(DiagnosticMessage::UnusedImport {
                name: import.name.clone(),
                span: import.span,
            });
        }
    }
}
