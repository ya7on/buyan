use std::{collections::HashSet, path::PathBuf};

use crate::{
    common::CompileContext,
    error::{DiagnosticMessage, Diagnostics},
    fs::{FileSystem, Module},
    pipeline::{Stage, StageResult},
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
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();
        let Some(entrypoint) = self.file_loader.read(&input) else {
            diagnostics.emit_fatal(DiagnosticMessage::FileNotFound {
                path: input.display().to_string(),
            });
            return StageResult::new(None, diagnostics);
        };
        let mut visited = HashSet::from([entrypoint.absolute.clone()]);
        let mut queue = vec![entrypoint];
        let mut modules = Vec::new();

        while let Some(module) = queue.pop() {
            let source_id = context.add_source(module.absolute.clone(), module.content.clone());
            let content_len = module.content.len();
            let lexer_result = match lex(&LexInput {
                content: module.content,
                source_id,
            }) {
                Ok(tokens) => tokens,
                Err(err) => {
                    for error in err {
                        diagnostics.emit_fatal(error);
                    }
                    continue;
                }
            };
            let parse_result = match parse(ParserInput {
                tokens: lexer_result.tokens,
                source_id,
                content_len,
            }) {
                Ok(ast) => ast,
                Err(err) => {
                    for error in err {
                        diagnostics.emit_fatal(error);
                    }
                    continue;
                }
            };

            for import in &parse_result.ast.imports {
                if import.first() == Some("std") {
                    match import.to_string().as_str() {
                        "std.stack" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/stack.by"),
                                content: include_str!("../../../stdlib/stack.by").to_string(),
                                name: "std.stack".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.math" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/math.by"),
                                content: include_str!("../../../stdlib/math.by").to_string(),
                                name: "std.math".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.cfg" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/cfg.by"),
                                content: include_str!("../../../stdlib/cfg.by").to_string(),
                                name: "std.cfg".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.io" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/io.by"),
                                content: include_str!("../../../stdlib/io.by").to_string(),
                                name: "std.io".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.str" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/str.by"),
                                content: include_str!("../../../stdlib/str.by").to_string(),
                                name: "std.str".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.ptr" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/ptr.by"),
                                content: include_str!("../../../stdlib/ptr.by").to_string(),
                                name: "std.ptr".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.unsafe.mem" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/unsafe/mem.by"),
                                content: include_str!("../../../stdlib/unsafe/mem.by").to_string(),
                                name: "std.unsafe.mem".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        _ => {
                            diagnostics.emit_fatal(DiagnosticMessage::ImportError {
                                path: import.to_string(),
                                span: import.span,
                            });
                        }
                    }
                    continue;
                }

                let path = Into::<PathBuf>::into(format!(
                    "{}.by",
                    import.value.to_string().replace('.', "/")
                ));
                let Some(module) = self.file_loader.read(&path) else {
                    diagnostics.emit_fatal(DiagnosticMessage::ImportError {
                        path: path.display().to_string(),
                        span: import.span,
                    });
                    continue;
                };
                if visited.insert(module.absolute.clone()) {
                    queue.push(module);
                }
            }

            modules.push(parse_result.ast);
        }

        StageResult::new(Some(ASTProgram { modules }), diagnostics)
    }
}
