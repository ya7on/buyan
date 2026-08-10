use std::{collections::HashSet, path::PathBuf};

use crate::{
    common::{CompileContext, CompileTarget, Spanned},
    error::{DiagnosticMessage, Diagnostics},
    fs::{FileSystem, Module},
    pipeline::{Stage, StageResult},
    stages::parse::{
        ast::{ASTAttribute, ASTProgram},
        lexer::{LexInput, lex},
        parser::{ParserInput, parse},
    },
};

fn target_matches(
    attributes: &[Spanned<ASTAttribute>],
    target: CompileTarget,
) -> Result<bool, DiagnosticMessage> {
    let targets = attributes
        .iter()
        .filter(|attribute| attribute.value.name == "target")
        .map(|attribute| {
            let Some(value) = attribute.value.value.as_deref() else {
                return Err(DiagnosticMessage::InvalidAttribute {
                    name: "target".to_string(),
                    reason: "expected #[target = \"<target>\"]".to_string(),
                    span: attribute.span,
                });
            };
            CompileTarget::try_from(value).map_err(|()| DiagnosticMessage::InvalidAttribute {
                name: "target".to_string(),
                reason: format!("unknown target \"{value}\";"),
                span: attribute.span,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(targets.is_empty() || targets.contains(&target))
}

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
            let mut module = match parse(ParserInput {
                tokens: lexer_result.tokens,
                source_id,
                content_len,
            }) {
                Ok(parse_result) => parse_result.ast,
                Err(err) => {
                    for error in err {
                        diagnostics.emit_fatal(error);
                    }
                    continue;
                }
            };
            module.imports = std::mem::take(&mut module.imports)
                .into_iter()
                .filter_map(|mut import| {
                    let matches_target =
                        match target_matches(&import.value.attributes, context.target) {
                            Ok(matches_target) => matches_target,
                            Err(diagnostic) => {
                                diagnostics.emit_fatal(diagnostic);
                                return None;
                            }
                        };
                    if !matches_target {
                        return None;
                    }
                    import
                        .value
                        .attributes
                        .retain(|attribute| attribute.value.name != "target");
                    Some(import)
                })
                .collect();
            module.words = std::mem::take(&mut module.words)
                .into_iter()
                .filter_map(|mut word| {
                    let matches_target =
                        match target_matches(&word.value.attributes, context.target) {
                            Ok(matches_target) => matches_target,
                            Err(diagnostic) => {
                                diagnostics.emit_fatal(diagnostic);
                                return None;
                            }
                        };
                    if !matches_target {
                        return None;
                    }
                    word.value
                        .attributes
                        .retain(|attribute| attribute.value.name != "target");
                    Some(word)
                })
                .collect();

            for import in &module.imports {
                if import.value.name.first() == Some("std") {
                    match import.value.name.to_string().as_str() {
                        "std.intrinsics" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/intrinsics.by"),
                                content: include_str!("../../../stdlib/intrinsics.by").to_string(),
                                name: "std.intrinsics".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
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
                        "std.u8" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/u8.by"),
                                content: include_str!("../../../stdlib/u8.by").to_string(),
                                name: "std.u8".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.u16" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/u16.by"),
                                content: include_str!("../../../stdlib/u16.by").to_string(),
                                name: "std.u16".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.usize" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/usize.by"),
                                content: include_str!("../../../stdlib/usize.by").to_string(),
                                name: "std.usize".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.bool" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/bool.by"),
                                content: include_str!("../../../stdlib/bool.by").to_string(),
                                name: "std.bool".to_string(),
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
                        "std.bytearray" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/bytearray.by"),
                                content: include_str!("../../../stdlib/bytearray.by").to_string(),
                                name: "std.bytearray".to_string(),
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
                        "std.os.cpm" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/os/cpm.by"),
                                content: include_str!("../../../stdlib/os/cpm.by").to_string(),
                                name: "std.os.cpm".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        "std.os.interpreter" => {
                            let module = Module {
                                absolute: PathBuf::from("stdlib/os/interpreter.by"),
                                content: include_str!("../../../stdlib/os/interpreter.by")
                                    .to_string(),
                                name: "std.os.interpreter".to_string(),
                            };
                            if visited.insert(module.absolute.clone()) {
                                queue.push(module);
                            }
                        }
                        _ => {
                            diagnostics.emit_fatal(DiagnosticMessage::ImportError {
                                path: import.value.name.to_string(),
                                span: import.span,
                            });
                        }
                    }
                    continue;
                }

                let path = Into::<PathBuf>::into(format!(
                    "{}.by",
                    import.value.name.to_string().replace('.', "/")
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

            modules.push(module);
        }

        StageResult::new(Some(ASTProgram { modules }), diagnostics)
    }
}
