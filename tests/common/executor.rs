#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

use buyan::{
    error::CompileError,
    fs::FileSystem,
    pipeline::PipelineBuilder,
    stages::{
        lower::{collect::CollectSymbolsStage, ir::IRProgram, stage::LowerStage},
        parse::{
            ast::{ASTModule, ASTProgram},
            lexer::TokenKind,
            stage::ParseStage,
        },
        semantic::{
            collect_hir::CollectHIRStage, collect_names::CollectNamesStage, hir::HIRProgram,
            type_check::TypeCheckStage,
        },
    },
};
use chumsky::span::SimpleSpan;

type LexerResult = Result<Vec<(TokenKind, SimpleSpan)>, Vec<CompileError>>;
type ParserResult = Result<ASTModule, Vec<CompileError>>;

#[derive(Default)]
pub struct TestFilesystem {
    pub files: HashMap<String, String>,
}

impl FileSystem for TestFilesystem {
    fn read(&self, path: &std::path::Path) -> Option<buyan::fs::Module> {
        self.files
            .get(path.to_str().unwrap())
            .map(|content| buyan::fs::Module {
                absolute: path.to_path_buf(),
                content: content.to_string(),
                name: path.to_str().unwrap().to_string(),
            })
    }
}

#[derive(Default)]
pub struct TestExecutor {
    pub inputs: HashMap<String, String>,
    pub ast: Option<Result<ASTProgram, Vec<CompileError>>>,
    pub hir: Option<Result<HIRProgram, Vec<CompileError>>>,
    pub ir: Option<Result<IRProgram, Vec<CompileError>>>,
}

impl TestExecutor {
    pub fn input<P: ToString, C: ToString>(input: (P, C)) -> Self {
        let (path, content) = input;
        Self {
            inputs: vec![(path.to_string(), content.to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        }
    }

    pub fn add_file<P: ToString, C: ToString>(mut self, input: (P, C)) -> Self {
        let (path, content) = input;
        self.inputs.insert(path.to_string(), content.to_string());
        self
    }

    pub fn check(mut self) -> Self {
        let pipeline = PipelineBuilder::new(PathBuf::from("app.by"));
        let pipeline = pipeline.stage_initialized::<ParseStage<TestFilesystem>>(ParseStage {
            file_loader: TestFilesystem {
                files: self.inputs.clone(),
            },
        });
        self.ast = Some(pipeline.dump().cloned().map_err(|err| err.clone()));
        let pipeline = pipeline
            .stage::<CollectNamesStage>()
            .stage::<CollectHIRStage>()
            .stage::<TypeCheckStage>();
        self.hir = Some(
            pipeline
                .dump()
                .map(|(_ctx, hir)| hir.clone())
                .map_err(|err| err.clone()),
        );
        let pipeline = pipeline
            .stage::<CollectSymbolsStage>()
            .stage::<LowerStage>();
        self.ir = Some(
            pipeline
                .dump()
                .map(|(_ctx, ir)| ir.clone())
                .map_err(|err| err.clone()),
        );
        self
    }
}
