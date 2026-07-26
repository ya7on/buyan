use std::collections::{HashMap, HashSet};

use crate::{
    common::{CompileContext, DottedPath, Spanned},
    error::CompileError,
    pipeline::Stage,
    stages::{
        parse::ast::{ASTProgram, ASTStruct},
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

fn register_struct(
    name: &DottedPath,
    declarations: &HashMap<DottedPath, StructDeclaration<'_>>,
    visiting: &mut HashSet<DottedPath>,
    visited: &mut HashSet<DottedPath>,
    context: &mut HIRContext,
) -> Result<SymbolId, CompileError> {
    let declaration = declarations
        .get(name)
        .ok_or_else(|| CompileError::Unknown {
            label: format!("Struct declaration not found: {name}"),
        })?;

    if visited.contains(name) {
        return context
            .lookup(name)
            .ok_or_else(|| CompileError::SymbolNotFound {
                name: name.to_string(),
                span: declaration.item.name.span,
            });
    }

    visiting.insert(name.clone());

    let mut fields = Vec::with_capacity(declaration.item.fields.len());
    for field in &declaration.item.fields {
        let ty = if let Some(dependency) = (field.len() == 1)
            .then(|| declaration.module_name.extend(field))
            .as_ref()
            .into_iter()
            .chain(std::iter::once(&field.value))
            .find(|name| declarations.contains_key(*name))
        {
            if visiting.contains(dependency) {
                return Err(CompileError::RecursiveStruct {
                    name: dependency.to_string(),
                    span: field.span,
                });
            }
            HIRType::Struct(register_struct(
                dependency,
                declarations,
                visiting,
                visited,
                context,
            )?)
        } else {
            let Some((id, SymbolKind::Type { .. })) = context.lookup_and_get(&field.value) else {
                return Err(CompileError::SymbolNotFound {
                    name: field.value.to_string(),
                    span: field.span,
                });
            };
            HIRType::BuiltIn(id)
        };
        fields.push(Spanned::new(ty, field.span));
    }

    let module_id =
        context
            .lookup(&declaration.module_name)
            .ok_or_else(|| CompileError::SymbolNotFound {
                name: declaration.module_name.to_string(),
                span: declaration.item.name.span,
            })?;
    let struct_id = context.register_struct(module_id, declaration.item, fields)?;
    visiting.remove(name);
    visited.insert(name.clone());
    Ok(struct_id)
}

fn register_structs(input: &ASTProgram, context: &mut HIRContext) -> Result<(), Vec<CompileError>> {
    let mut declarations = HashMap::new();
    let mut errors = Vec::new();

    for module in &input.modules {
        for item in &module.structs {
            let name = module.name.append(item.name.as_str());
            if declarations.contains_key(&name) || context.lookup(&name).is_some() {
                errors.push(CompileError::SymbolAlreadyExists {
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

    if !errors.is_empty() {
        return Err(errors);
    }

    let names = declarations.keys().cloned().collect::<Vec<_>>();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for name in names {
        if !visited.contains(&name)
            && let Err(err) =
                register_struct(&name, &declarations, &mut visiting, &mut visited, context)
        {
            return Err(vec![err]);
        }
    }
    Ok(())
}

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
            if let Err(err) = context.register_module(module) {
                errors.push(err);
            }
        }

        if errors.is_empty()
            && let Err(struct_errors) = register_structs(&input, &mut context)
        {
            errors.extend(struct_errors);
        }

        for module in &input.modules {
            for word in &module.words {
                match context.register_word(&module.name, word) {
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
