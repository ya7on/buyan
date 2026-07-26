use std::path::PathBuf;

use ariadne::{Color, Label, Report, ReportKind};
use buyan::{
    common::CompileContext,
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

fn print_errors(context: &CompileContext, errors: &[CompileError]) {
    let mut cache = ariadne::sources(
        context
            .sources
            .values()
            .map(|source| (source.path.display().to_string(), source.content.clone())),
    );

    for error in errors {
        let Some(span) = error.span() else {
            eprintln!("{error:?}");
            continue;
        };
        let Some(source) = context.sources.get(&span.source_id) else {
            eprintln!("{error:?}");
            continue;
        };

        let path = source.path.display().to_string();
        let range = source.content[..span.start].chars().count()
            ..source.content[..span.end].chars().count();

        if let Err(error) = Report::build(ReportKind::Error, (path.clone(), range.clone()))
            .with_message(format!("{error:?}"))
            .with_label(Label::new((path, range)).with_color(Color::Red))
            .finish()
            .eprint(&mut cache)
        {
            eprintln!("failed to print diagnostic: {error}");
        }
    }
}

fn main() {
    let args = Args::parse();

    let pipeline = PipelineBuilder::new(args.path)
        .stage(ParseStage::<RealFileSystem>::default())
        // .stage(DumpAst)
        .stage(CollectNamesStage)
        .stage(CollectHIRStage)
        .stage(TypeCheckStage)
        .stage(CollectSymbolsStage)
        .stage(LowerStage)
        .stage(IRInterpreter::default());

    if let Err(errors) = pipeline.dump() {
        print_errors(&pipeline.context, errors);
    }
}
