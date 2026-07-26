use std::path::PathBuf;

use ariadne::{Color, Label, Report, ReportKind};
use buyan::{
    common::CompileContext,
    error::{Diagnostic, DiagnosticKind},
    fs::RealFileSystem,
    lints::{LintPass, empty_word::EmptyWord},
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

fn print_errors(context: &CompileContext, diagnostics: &[Diagnostic]) {
    let mut cache = ariadne::sources(
        context
            .sources
            .values()
            .map(|source| (source.path.display().to_string(), source.content.clone())),
    );

    for diagnostic in diagnostics {
        let code = format!("B{:04}", diagnostic.code);
        let Some(span) = diagnostic.message.span() else {
            eprintln!("[{code}] {:?}", diagnostic.message);
            continue;
        };
        let Some(source) = context.sources.get(&span.source_id) else {
            eprintln!("[{code}] {:?}", diagnostic.message);
            continue;
        };

        let path = source.path.display().to_string();
        let range = source.content[..span.start].chars().count()
            ..source.content[..span.end].chars().count();
        let (kind, color) = match diagnostic.kind {
            DiagnosticKind::Error => (ReportKind::Error, Color::Red),
            DiagnosticKind::Warning => (ReportKind::Warning, Color::Yellow),
        };

        if let Err(error) = Report::build(kind, (path.clone(), range.clone()))
            .with_code(code)
            .with_message(format!("{:?}", diagnostic.message))
            .with_label(Label::new((path, range)).with_color(color))
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
        .stage(LintPass::default().lint(EmptyWord))
        .stage(CollectSymbolsStage)
        .stage(LowerStage);

    print_errors(&pipeline.context, &pipeline.diagnostics.items);

    pipeline.stage(IRInterpreter::default());
}
