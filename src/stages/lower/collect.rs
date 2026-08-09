use crate::{
    common::{CompileContext, CompileTarget, DottedPath},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        lower::{context::IRContext, ir::IRType},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::{HIRProgram, HIRType},
        },
    },
};

#[derive(Debug, Default)]
pub struct CollectSymbolsStage;

impl CollectSymbolsStage {
    fn collect_struct_type(
        hir_ctx: &HIRContext,
        target: CompileTarget,
        ty: &HIRType,
    ) -> Option<IRType> {
        match ty {
            HIRType::BuiltIn(symbol_id) => match hir_ctx.get(*symbol_id) {
                Some(SymbolKind::Type { name, .. }) => IRType::from_builtin_type(target, name),
                _ => None,
            },
            HIRType::Struct(symbol_id) => match hir_ctx.get(*symbol_id) {
                Some(SymbolKind::Struct { fields, .. }) => Some(IRType::Struct {
                    fields: fields
                        .iter()
                        .map(|field| Self::collect_struct_type(hir_ctx, target, &field.value))
                        .collect::<Option<Vec<_>>>()?,
                }),
                _ => None,
            },
            HIRType::Lambda { .. } => Some(IRType::Lambda),
            HIRType::TypeVar(_) | HIRType::StackVar(_) => None,
        }
    }
}

impl Stage<CompileContext> for CollectSymbolsStage {
    type Input = (HIRContext, HIRProgram);
    type Output = (IRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, hir_program): Self::Input,
        context: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut ir_ctx = IRContext::default();
        for name in ["bool", "u8", "u16", "ptr", "usize"] {
            let Some(ty) = IRType::from_builtin_type(context.target, name) else {
                diagnostics.emit_fatal(DiagnosticMessage::Unknown {
                    label: format!("built-in type '{name}' is not supported by this target"),
                });
                continue;
            };
            let Some(symbol_id) = hir_ctx.lookup(&DottedPath::parse(name)) else {
                diagnostics.emit_fatal(DiagnosticMessage::Unknown {
                    label: format!("built-in type '{name}' not found"),
                });
                continue;
            };
            ir_ctx.register_type(symbol_id, ty);
        }

        for module in &hir_program.modules {
            for item in &module.structs {
                let Some(ty) =
                    Self::collect_struct_type(&hir_ctx, context.target, &HIRType::Struct(item.id))
                else {
                    diagnostics.emit_fatal(DiagnosticMessage::CannotInferType {
                        label: format!("cannot collect type for '{}'", item.name.value),
                        span: item.span,
                    });
                    continue;
                };
                ir_ctx.register_type(item.id, ty);
            }
        }

        StageResult::new(Some((ir_ctx, hir_program)), diagnostics)
    }
}
