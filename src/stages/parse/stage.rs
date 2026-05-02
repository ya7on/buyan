use std::{collections::HashSet, path::PathBuf};

use crate::{
    common::CompileContext,
    error::CompileError,
    fs::{FileSystem, Module},
    pipeline::Stage,
    stages::parse::{
        ast::ASTProgram,
        lexer::{LexInput, lex},
        parser::{ParserInput, parse},
    },
};

#[derive(Debug, Default)]
pub struct ParseStage<F: FileSystem> {
    pub file_loader: F,
}

impl<F: FileSystem> Stage<CompileContext> for ParseStage<F> {
    type Input = PathBuf;
    type Output = ASTProgram;

    fn execute(
        &mut self,
        input: Self::Input,
        context: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let entrypoint = self.file_loader.read(&input).ok_or_else(|| {
            vec![CompileError::FileNotFound {
                path: input.display().to_string(),
            }]
        })?;
        context
            .sources
            .insert(entrypoint.absolute.clone(), entrypoint.content.clone());

        let mut queue = vec![entrypoint];
        let mut errors = Vec::new();
        let mut modules = Vec::new();
        let mut visited = HashSet::new();

        while let Some(module) = queue.pop() {
            let lexer_result = match lex(LexInput {
                content: module.content,
            }) {
                Ok(tokens) => tokens,
                Err(err) => {
                    errors.extend(err);
                    continue;
                }
            };
            let parse_result = match parse(ParserInput {
                tokens: lexer_result.tokens,
            }) {
                Ok(ast) => ast,
                Err(err) => {
                    errors.extend(err);
                    continue;
                }
            };

            for import in &parse_result.ast.imports {
                if import.first() == Some("std") {
                    match import.to_string().as_str() {
                        "std.stack" => {
                            queue.push(Module {
                                absolute: PathBuf::from("stdlib/stack.by"),
                                content: include_str!("../../../stdlib/stack.by").to_string(),
                                name: "std.stack".to_string(),
                            });
                        }
                        "std.math" => {
                            queue.push(Module {
                                absolute: PathBuf::from("stdlib/math.by"),
                                content: include_str!("../../../stdlib/math.by").to_string(),
                                name: "std.math".to_string(),
                            });
                        }
                        "std.cfg" => {
                            queue.push(Module {
                                absolute: PathBuf::from("stdlib/cfg.by"),
                                content: include_str!("../../../stdlib/cfg.by").to_string(),
                                name: "std.cfg".to_string(),
                            });
                        }
                        "std.io" => {
                            queue.push(Module {
                                absolute: PathBuf::from("stdlib/io.by"),
                                content: include_str!("../../../stdlib/io.by").to_string(),
                                name: "std.io".to_string(),
                            });
                        }
                        _ => {
                            errors.push(CompileError::ImportError {
                                path: import.to_string(),
                                span: import.span,
                            });
                        }
                    }
                    continue;
                }

                let path = Into::<PathBuf>::into(format!(
                    "{}.by",
                    import.value.to_string().replace(".", "/")
                ));
                if !visited.insert(path.clone()) {
                    continue;
                }
                let Some(module) = self.file_loader.read(&path) else {
                    errors.push(CompileError::ImportError {
                        path: path.display().to_string(),
                        span: import.span,
                    });
                    continue;
                };
                context
                    .sources
                    .insert(module.absolute.clone(), module.content.clone());
                queue.push(module);
            }

            modules.push(parse_result.ast);
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(ASTProgram { modules })
    }
}
