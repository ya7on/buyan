use std::collections::HashMap;

use crate::{
    common::{DottedPath, Spanned},
    error::DiagnosticMessage,
    stages::{
        parse::ast::{ASTModule, ASTStackEffectItem, ASTStruct, ASTWord, ASTWordVar},
        semantic::hir::HIRType,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

impl SymbolId {
    #[must_use]
    pub const fn id(&self) -> usize {
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
        entrypoint: bool,
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
    },
    Type {
        name: String,
    },
    Struct {
        name: String,
        fields: Vec<Spanned<HIRType>>,
    },
}

#[derive(Debug)]
pub struct HIRContext {
    pub symbols_index: HashMap<String, SymbolId>,
    pub symbols: Vec<SymbolKind>,
}

impl HIRContext {
    pub(crate) fn new() -> Result<Self, DiagnosticMessage> {
        let mut result = Self {
            symbols_index: HashMap::new(),
            symbols: Vec::new(),
        };

        // bool
        result
            .register(
                &DottedPath::parse("bool"),
                SymbolKind::Type {
                    name: "bool".to_string(),
                },
            )
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: "failed to register built-in type 'bool'".to_string(),
            })?;

        // u16
        result
            .register(
                &DottedPath::parse("u16"),
                SymbolKind::Type {
                    name: "u16".to_string(),
                },
            )
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: "failed to register built-in type 'u16'".to_string(),
            })?;

        // ptr
        result
            .register(
                &DottedPath::parse("ptr"),
                SymbolKind::Type {
                    name: "ptr".to_string(),
                },
            )
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: "failed to register built-in type 'ptr'".to_string(),
            })?;

        // usize
        result
            .register(
                &DottedPath::parse("usize"),
                SymbolKind::Type {
                    name: "usize".to_string(),
                },
            )
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: "failed to register built-in type 'usize'".to_string(),
            })?;

        // u8
        result
            .register(
                &DottedPath::parse("u8"),
                SymbolKind::Type {
                    name: "u8".to_string(),
                },
            )
            .ok_or_else(|| DiagnosticMessage::Unknown {
                label: "failed to register built-in type 'u8'".to_string(),
            })?;

        Ok(result)
    }

    #[must_use]
    pub fn format_type(&self, ty: &HIRType) -> String {
        match ty {
            HIRType::BuiltIn(symbol_id) => match self.get(*symbol_id) {
                Some(SymbolKind::Type { name, .. }) => name.clone(),
                _ => "<unknown>".to_string(),
            },
            HIRType::Struct(symbol_id) => match self.get(*symbol_id) {
                Some(SymbolKind::Struct { name, .. }) => name.clone(),
                _ => "<unknown>".to_string(),
            },
            HIRType::TypeVar(symbol_id) => match self.get(*symbol_id) {
                Some(SymbolKind::TypeVar { name, .. }) => name.clone(),
                _ => "<unknown>".to_string(),
            },
            HIRType::StackVar(symbol_id) => match self.get(*symbol_id) {
                Some(SymbolKind::StackVar { name }) => format!("...{name}"),
                _ => "<unknown>".to_string(),
            },
            HIRType::Lambda {
                stack_in,
                stack_out,
            } => format!(
                "|{} -- {}|",
                stack_in
                    .iter()
                    .map(|ty| self.format_type(ty))
                    .collect::<Vec<_>>()
                    .join(", "),
                stack_out
                    .iter()
                    .map(|ty| self.format_type(ty))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    #[must_use]
    pub fn lookup(&self, name: &DottedPath) -> Option<SymbolId> {
        self.symbols_index.get(&name.to_string()).copied()
    }

    #[must_use]
    pub fn lookup_and_get(&self, name: &DottedPath) -> Option<(SymbolId, &SymbolKind)> {
        let id = self.lookup(name)?;
        let kind = self.get(id)?;
        Some((id, kind))
    }

    #[must_use]
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

    pub(crate) fn register_module(
        &mut self,
        module: &ASTModule,
    ) -> Result<SymbolId, DiagnosticMessage> {
        let module_name = module.name.to_string();
        let Some(id) = self.register(
            &module.name,
            SymbolKind::Module {
                name: module.name.value.clone(),
            },
        ) else {
            return Err(DiagnosticMessage::SymbolAlreadyExists {
                name: module_name,
                span: module.name.span,
            });
        };
        Ok(id)
    }

    pub(crate) fn register_word(
        &mut self,
        module_name: &DottedPath,
        word: &ASTWord,
        entrypoint: bool,
    ) -> Result<SymbolId, DiagnosticMessage> {
        if !matches!(
            self.lookup_and_get(module_name),
            Some((_, SymbolKind::Module { .. }))
        ) {
            return Err(DiagnosticMessage::Unknown {
                label: "Invalid module for word".to_string(),
            });
        }
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
                        .ok_or_else(|| DiagnosticMessage::SymbolAlreadyExists {
                            name: name.to_string(),
                            span: word.name.span,
                        })?;
                    stackvars.push(Spanned::new(typevar_id, name.span));
                }
                ASTWordVar::Type { name } => {
                    let fullpath = wordpath.append(name);
                    let typevar_id = self
                        .register(
                            &fullpath,
                            SymbolKind::TypeVar {
                                name: name.to_string(),
                            },
                        )
                        .ok_or_else(|| DiagnosticMessage::SymbolAlreadyExists {
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
                self.handle_stack_item(module_name, &wordpath, item)?,
                item.span,
            ));
        }
        let mut stack_out = Vec::new();
        for item in &word.stack_effect.stack_out {
            stack_out.push(Spanned::new(
                self.handle_stack_item(module_name, &wordpath, item)?,
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
                entrypoint,
            },
        )
        .ok_or_else(|| DiagnosticMessage::SymbolAlreadyExists {
            name: word.name.to_string(),
            span: word.name.span,
        })
    }

    pub(crate) fn register_struct(
        &mut self,
        module_id: SymbolId,
        item: &ASTStruct,
        fields: Vec<Spanned<HIRType>>,
    ) -> Result<SymbolId, DiagnosticMessage> {
        let Some(SymbolKind::Module { name: module_name }) = self.get(module_id) else {
            return Err(DiagnosticMessage::Unknown {
                label: "Invalid module for struct".to_string(),
            });
        };
        let fullpath = module_name.append(item.name.as_str());
        let display_name = fullpath.to_string();
        self.register(
            &fullpath,
            SymbolKind::Struct {
                name: display_name.clone(),
                fields,
            },
        )
        .ok_or(DiagnosticMessage::SymbolAlreadyExists {
            name: display_name,
            span: item.name.span,
        })
    }

    pub(crate) fn handle_stack_item(
        &self,
        module_name: &DottedPath,
        wordpath: &DottedPath,
        item: &Spanned<ASTStackEffectItem>,
    ) -> Result<HIRType, DiagnosticMessage> {
        match &item.value {
            ASTStackEffectItem::Symbol { name } => {
                if name.len() == 1
                    && let Some((id, SymbolKind::TypeVar { .. })) =
                        self.lookup_and_get(&wordpath.extend(name))
                {
                    return Ok(HIRType::TypeVar(id));
                }

                (name.len() == 1)
                    .then(|| module_name.extend(name))
                    .as_ref()
                    .into_iter()
                    .chain(std::iter::once(name))
                    .find_map(|name| {
                        let (id, kind) = self.lookup_and_get(name)?;
                        match kind {
                            SymbolKind::Type { .. } => Some(HIRType::BuiltIn(id)),
                            SymbolKind::Struct { .. } => Some(HIRType::Struct(id)),
                            _ => None,
                        }
                    })
                    .ok_or_else(|| DiagnosticMessage::SymbolNotFound {
                        name: name.to_string(),
                        span: item.span,
                    })
            }
            ASTStackEffectItem::StackVar { name } => {
                let Some((id, SymbolKind::StackVar { .. })) =
                    self.lookup_and_get(&wordpath.append(name))
                else {
                    return Err(DiagnosticMessage::SymbolNotFound {
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
                    .map(|item| self.handle_stack_item(module_name, wordpath, item))
                    .collect::<Result<Vec<_>, _>>()?;
                let stack_out = stack_effect
                    .stack_out
                    .iter()
                    .map(|item| self.handle_stack_item(module_name, wordpath, item))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(HIRType::Lambda {
                    stack_in,
                    stack_out,
                })
            }
        }
    }
}
