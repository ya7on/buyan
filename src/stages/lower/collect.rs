use crate::{
    common::{CompileContext, DottedPath},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        lower::{context::IRContext, ir::IRType},
        semantic::{
            context::{HIRContext, SymbolKind},
            hir::{HIRConst, HIRProgram, HIRType},
        },
    },
};

#[derive(Debug, Default)]
pub struct CollectSymbolsStage;

impl CollectSymbolsStage {
    fn collect_struct_type(hir_ctx: &HIRContext, ty: &HIRType) -> Option<IRType> {
        match ty {
            HIRType::BuiltIn(symbol_id) => match hir_ctx.get(*symbol_id) {
                Some(SymbolKind::Type { name, .. }) => match name.as_str() {
                    "bool" => Some(IRType::Bool),
                    "u8" => Some(IRType::U8),
                    "string" => Some(IRType::String),
                    _ => None,
                },
                _ => None,
            },
            HIRType::Struct(symbol_id) => match hir_ctx.get(*symbol_id) {
                Some(SymbolKind::Struct { fields, .. }) => Some(IRType::Struct {
                    fields: fields
                        .iter()
                        .map(|field| Self::collect_struct_type(hir_ctx, &field.value))
                        .collect::<Option<Vec<_>>>()?,
                }),
                _ => None,
            },
            HIRType::Array { element_type, size } => {
                let HIRConst::Value(size) = size else {
                    return None;
                };
                Some(IRType::Array {
                    element_type: Box::new(Self::collect_struct_type(hir_ctx, element_type)?),
                    size: *size,
                })
            }
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
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut ir_ctx = IRContext::default();
        for (name, ty) in [
            ("bool", IRType::Bool),
            ("u8", IRType::U8),
            ("string", IRType::String),
        ] {
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
                let Some(ty) = Self::collect_struct_type(&hir_ctx, &HIRType::Struct(item.id))
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
