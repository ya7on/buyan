use std::collections::{HashMap, HashSet};

use crate::{
    common::{CompileContext, DottedPath, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    pipeline::{Stage, StageResult},
    stages::{
        parse::ast::{ASTProgram, ASTStackEffectItem, ASTStruct},
        semantic::{
            context::{HIRContext, SymbolId, SymbolKind},
            hir::HIRType,
        },
    },
};

struct StructDeclaration<'a> {
    module_name: DottedPath,
    item: &'a ASTStruct,
}

#[derive(Debug, Default)]
pub struct CollectNamesStage;

impl CollectNamesStage {
    fn register_struct_field(
        field: &Spanned<ASTStackEffectItem>,
        module_name: &DottedPath,
        declarations: &HashMap<DottedPath, StructDeclaration<'_>>,
        visiting: &mut HashSet<DottedPath>,
        visited: &mut HashSet<DottedPath>,
        context: &mut HIRContext,
    ) -> Result<HIRType, DiagnosticMessage> {
        match &field.value {
            ASTStackEffectItem::Symbol { name } => {
                if let Some(dependency) = (name.len() == 1)
                    .then(|| module_name.extend(name))
                    .as_ref()
                    .into_iter()
                    .chain(std::iter::once(name))
                    .find(|name| declarations.contains_key(*name))
                {
                    if visiting.contains(dependency) {
                        return Err(DiagnosticMessage::RecursiveStruct {
                            name: dependency.to_string(),
                            span: field.span,
                        });
                    }
                    return Ok(HIRType::Struct(Self::register_struct(
                        dependency,
                        declarations,
                        visiting,
                        visited,
                        context,
                    )?));
                }
                let Some((id, SymbolKind::Type { .. })) = context.lookup_and_get(name) else {
                    return Err(DiagnosticMessage::SymbolNotFound {
                        name: name.to_string(),
                        span: field.span,
                    });
                };
                Ok(HIRType::BuiltIn(id))
            }
            ASTStackEffectItem::StackVar { .. } | ASTStackEffectItem::Lambda { .. } => {
                // TODO: generics and lambdas
                Err(DiagnosticMessage::InvalidSymbol {
                    name: "stack variables and lambdas are not supported in struct fields"
                        .to_string(),
                    span: field.span,
                })
            }
        }
    }

    fn register_struct(
        name: &DottedPath,
        declarations: &HashMap<DottedPath, StructDeclaration<'_>>,
        visiting: &mut HashSet<DottedPath>,
        visited: &mut HashSet<DottedPath>,
        context: &mut HIRContext,
    ) -> Result<SymbolId, DiagnosticMessage> {
        let declaration = declarations
            .get(name)
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: format!("Struct declaration not found: {name}"),
            })?;

        if visited.contains(name) {
            return context
                .lookup(name)
                .ok_or_else(|| DiagnosticMessage::SymbolNotFound {
                    name: name.to_string(),
                    span: declaration.item.name.span,
                });
        }

        visiting.insert(name.clone());

        let mut fields = Vec::with_capacity(declaration.item.fields.len());
        for field in &declaration.item.fields {
            let ty = Self::register_struct_field(
                field,
                &declaration.module_name,
                declarations,
                visiting,
                visited,
                context,
            )?;
            fields.push(Spanned::new(ty, field.span));
        }

        let module_id = context.lookup(&declaration.module_name).ok_or_else(|| {
            DiagnosticMessage::SymbolNotFound {
                name: declaration.module_name.to_string(),
                span: declaration.item.name.span,
            }
        })?;
        let struct_id = context.register_struct(module_id, declaration.item, fields)?;
        visiting.remove(name);
        visited.insert(name.clone());
        Ok(struct_id)
    }

    fn register_structs(
        input: &ASTProgram,
        context: &mut HIRContext,
        diagnostics: &mut Diagnostics,
    ) {
        let mut declarations = HashMap::new();

        for module in &input.modules {
            for item in &module.structs {
                let name = module.name.append(item.name.as_str());
                if declarations.contains_key(&name) || context.lookup(&name).is_some() {
                    diagnostics.emit_fatal(DiagnosticMessage::SymbolAlreadyExists {
                        name: name.to_string(),
                        span: item.name.span,
                    });
                    continue;
                }
                declarations.insert(
                    name,
                    StructDeclaration {
                        module_name: module.name.value.clone(),
                        item,
                    },
                );
            }
        }

        let names = declarations.keys().cloned().collect::<Vec<_>>();
        let mut visited = HashSet::new();
        for name in names {
            if !visited.contains(&name) {
                let mut visiting = HashSet::new();
                if let Err(err) = Self::register_struct(
                    &name,
                    &declarations,
                    &mut visiting,
                    &mut visited,
                    context,
                ) {
                    diagnostics.emit_fatal(err);
                }
            }
        }
    }
}

impl Stage<CompileContext> for CollectNamesStage {
    type Input = ASTProgram;
    type Output = (HIRContext, ASTProgram);

    fn execute(&mut self, input: Self::Input, _: &mut CompileContext) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let mut context = match HIRContext::new() {
            Ok(context) => context,
            Err(error) => {
                diagnostics.emit_fatal(error);
                return StageResult::new(None, diagnostics);
            }
        };

        for module in &input.modules {
            if let Err(err) = context.register_module(module) {
                diagnostics.emit_fatal(err);
            }
        }

        Self::register_structs(&input, &mut context, &mut diagnostics);

        for (index, module) in input.modules.iter().enumerate() {
            for word in &module.words {
                match context.register_word(
                    &module.name,
                    word,
                    index == 0 && word.name.value == "main",
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        diagnostics.emit_fatal(err);
                    }
                }
            }
        }

        StageResult::new(Some((context, input)), diagnostics)
    }
}
