use std::collections::HashMap;

use crate::{
    common::{DottedPath, Spanned},
    error::CompileError,
    stages::{
        parse::ast::{ASTModule, ASTStackEffectItem, ASTWord, ASTWordVar},
        semantic::hir::HIRType,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

impl SymbolId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub enum SymbolKind {
    Module {
        name: DottedPath,
    },
    Word {
        typevars: Vec<Spanned<SymbolId>>,
        stackvars: Vec<Spanned<SymbolId>>,
        stack_in: Vec<Spanned<HIRType>>,
        stack_out: Vec<Spanned<HIRType>>,
    },
    Lambda {
        stack_in: Vec<HIRType>,
        stack_out: Vec<HIRType>,
    },
    StackVar {
        name: String,
    },
    TypeVar {
        name: String,
        traits: Vec<Spanned<SymbolId>>,
    },
    Trait {
        name: String,
    },
    Type {
        name: String,
        traits: Vec<SymbolId>,
    },
}

#[derive(Debug)]
pub struct HIRContext {
    pub symbols_index: HashMap<String, SymbolId>,
    pub symbols: Vec<SymbolKind>,
}

impl Default for HIRContext {
    fn default() -> Self {
        let mut result = Self {
            symbols_index: HashMap::new(),
            symbols: Vec::new(),
        };

        // traits
        let copy_trait_id = result
            .register(
                &DottedPath::parse("Copy"),
                SymbolKind::Trait {
                    name: "Copy".to_string(),
                },
            )
            .unwrap();
        let add_trait_id = result
            .register(
                &DottedPath::parse("Add"),
                SymbolKind::Trait {
                    name: "Add".to_string(),
                },
            )
            .unwrap();
        let sub_trait_id = result
            .register(
                &DottedPath::parse("Sub"),
                SymbolKind::Trait {
                    name: "Sub".to_string(),
                },
            )
            .unwrap();
        let mul_trait_id = result
            .register(
                &DottedPath::parse("Mul"),
                SymbolKind::Trait {
                    name: "Mul".to_string(),
                },
            )
            .unwrap();
        let div_trait_id = result
            .register(
                &DottedPath::parse("Div"),
                SymbolKind::Trait {
                    name: "Div".to_string(),
                },
            )
            .unwrap();
        let eq_trait_id = result
            .register(
                &DottedPath::parse("Eq"),
                SymbolKind::Trait {
                    name: "Eq".to_string(),
                },
            )
            .unwrap();
        let ord_trait_id = result
            .register(
                &DottedPath::parse("Ord"),
                SymbolKind::Trait {
                    name: "Ord".to_string(),
                },
            )
            .unwrap();

        // bool
        result.register(
            &DottedPath::parse("bool"),
            SymbolKind::Type {
                name: "bool".to_string(),
                traits: vec![copy_trait_id, eq_trait_id],
            },
        );

        // u8
        result
            .register(
                &DottedPath::parse("u8"),
                SymbolKind::Type {
                    name: "u8".to_string(),
                    traits: vec![
                        copy_trait_id,
                        add_trait_id,
                        sub_trait_id,
                        mul_trait_id,
                        div_trait_id,
                        eq_trait_id,
                        ord_trait_id,
                    ],
                },
            )
            .unwrap();

        // string
        result
            .register(
                &DottedPath::parse("string"),
                SymbolKind::Type {
                    name: "string".to_string(),
                    traits: vec![eq_trait_id],
                },
            )
            .unwrap();

        result
    }
}

impl HIRContext {
    pub fn lookup(&self, name: &DottedPath) -> Option<SymbolId> {
        self.symbols_index.get(&name.to_string()).copied()
    }

    pub fn lookup_and_get(&self, name: &DottedPath) -> Option<(SymbolId, &SymbolKind)> {
        let id = self.lookup(name)?;
        let kind = self.get(id)?;
        Some((id, kind))
    }

    pub fn get(&self, id: SymbolId) -> Option<&SymbolKind> {
        self.symbols.get(id.0)
    }

    fn register(&mut self, path: &DottedPath, kind: SymbolKind) -> Option<SymbolId> {
        let symbol_id = SymbolId(self.symbols.len());
        if self
            .symbols_index
            .insert(path.to_string(), symbol_id)
            .is_some()
        {
            return None;
        }
        self.symbols.push(kind);
        Some(symbol_id)
    }

    pub fn register_module(&mut self, module: &ASTModule) -> Result<SymbolId, CompileError> {
        let module_name = module.name.to_string();
        let Some(id) = self.register(
            &module.name,
            SymbolKind::Module {
                name: module.name.value.clone(),
            },
        ) else {
            return Err(CompileError::SymbolAlreadyExists {
                name: module_name,
                span: module.name.span,
            });
        };
        Ok(id)
    }

    pub fn register_word(
        &mut self,
        module_id: SymbolId,
        word: &ASTWord,
    ) -> Result<SymbolId, CompileError> {
        let Some(SymbolKind::Module { name: module_name }) = self.get(module_id) else {
            return Err(CompileError::Unknown {
                label: "Invalid module for word".to_string(),
            });
        };
        let wordpath = module_name.append(word.name.as_str()); // TODO FIXME

        let mut typevars = Vec::new();
        let mut stackvars = Vec::new();
        for var in &word.word_vars {
            match var {
                ASTWordVar::Stack { name } => {
                    let stackvar_path = wordpath.append(name.as_str());
                    let typevar_id = self
                        .register(
                            &stackvar_path,
                            SymbolKind::StackVar {
                                name: name.to_string(),
                            },
                        )
                        .ok_or_else(|| CompileError::SymbolAlreadyExists {
                            name: name.to_string(),
                            span: word.name.span,
                        })?;
                    stackvars.push(Spanned::new(typevar_id, name.span));
                }
                ASTWordVar::Type { name, traits } => {
                    let mut trait_ids = Vec::with_capacity(traits.len());
                    for trait_name in traits {
                        let Some((trait_id, SymbolKind::Trait { .. })) =
                            self.lookup_and_get(&DottedPath::parse(trait_name))
                        else {
                            return Err(CompileError::SymbolNotFound {
                                name: trait_name.value.clone(),
                                span: trait_name.span,
                            });
                        };
                        trait_ids.push(Spanned::new(trait_id, trait_name.span));
                    }

                    let fullpath = wordpath.append(name);
                    let typevar_id = self
                        .register(
                            &fullpath,
                            SymbolKind::TypeVar {
                                name: name.to_string(),
                                traits: trait_ids,
                            },
                        )
                        .ok_or_else(|| CompileError::SymbolAlreadyExists {
                            name: name.to_string(),
                            span: word.name.span,
                        })?;
                    typevars.push(Spanned::new(typevar_id, name.span));
                }
            }
        }

        let mut stack_in = Vec::new();
        for item in &word.stack_effect.stack_in {
            stack_in.push(Spanned::new(
                self.handle_stack_item(&wordpath, item)?,
                item.span,
            ));
        }
        let mut stack_out = Vec::new();
        for item in &word.stack_effect.stack_out {
            stack_out.push(Spanned::new(
                self.handle_stack_item(&wordpath, item)?,
                item.span,
            ));
        }

        self.register(
            &wordpath,
            SymbolKind::Word {
                stackvars,
                typevars,
                stack_in,
                stack_out,
            },
        )
        .ok_or_else(|| CompileError::SymbolAlreadyExists {
            name: word.name.to_string(),
            span: word.name.span,
        })
    }

    pub fn handle_stack_item(
        &self,
        wordpath: &DottedPath,
        item: &Spanned<ASTStackEffectItem>,
    ) -> Result<HIRType, CompileError> {
        match &item.value {
            ASTStackEffectItem::Symbol { name } => {
                'typevar: {
                    let Some((id, SymbolKind::TypeVar { .. })) =
                        self.lookup_and_get(&wordpath.append(name))
                    else {
                        break 'typevar;
                    };
                    return Ok(HIRType::TypeVar(id));
                };
                'global: {
                    let Some((id, SymbolKind::Type { .. })) =
                        self.lookup_and_get(&DottedPath::parse(name))
                    else {
                        break 'global;
                    };
                    return Ok(HIRType::BuiltIn(id));
                };
                Err(CompileError::SymbolNotFound {
                    name: name.to_owned(),
                    span: item.span,
                })
            }
            ASTStackEffectItem::StackVar { name } => {
                let Some((id, SymbolKind::StackVar { .. })) =
                    self.lookup_and_get(&wordpath.append(name))
                else {
                    return Err(CompileError::SymbolNotFound {
                        name: name.to_owned(),
                        span: item.span,
                    });
                };
                Ok(HIRType::StackVar(id))
            }
            ASTStackEffectItem::Lambda { stack_effect } => {
                let stack_in = stack_effect
                    .stack_in
                    .iter()
                    .map(|item| self.handle_stack_item(wordpath, item))
                    .collect::<Result<Vec<_>, _>>()?;
                let stack_out = stack_effect
                    .stack_out
                    .iter()
                    .map(|item| self.handle_stack_item(wordpath, item))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(HIRType::Lambda {
                    stack_in,
                    stack_out,
                })
            }
        }
    }
}
