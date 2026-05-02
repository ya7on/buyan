use std::path::PathBuf;

use buyan::{
    error::CompileError,
    fs::RealFileSystem,
    pipeline::PipelineBuilder,
    stages::{
        interpreter::executor::IRInterpreter,
        lower::{collect::CollectSymbolsStage, stage::LowerStage},
        parse::stage::ParseStage,
        semantic::{
            collect_hir::CollectHIRStage, collect_names::CollectNamesStage,
            type_check::TypeCheckStage,
        },
    },
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn print_errors(errors: &[CompileError]) {
    for error in errors {
        println!("{:?}", error);
    }
}

fn main() {
    let args = Args::parse();

    let pipeline = PipelineBuilder::new(args.path)
        .stage::<ParseStage<RealFileSystem>>()
        // .stage::<DumpAst>()
        .stage::<CollectNamesStage>()
        .stage::<CollectHIRStage>()
        .stage::<TypeCheckStage>()
        .stage::<CollectSymbolsStage>()
        .stage::<LowerStage>()
        .stage::<IRInterpreter>();
    // let context = pipeline.context();
    match pipeline.finish() {
        Ok(_) => {}
        Err(errors) => print_errors(&errors),
    }
}
