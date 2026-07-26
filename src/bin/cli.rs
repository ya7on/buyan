use std::path::PathBuf;

use ariadne::{Color, Label, Report, ReportKind};
use buyan::{
    common::CompileContext,
    error::{Diagnostic, DiagnosticKind},
    fs::RealFileSystem,
    lints::{LintPass, empty_word::EmptyWord, unused_import::UnusedImport},
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
        let title = diagnostic.message.title();
        let description = diagnostic.message.description();
        let help = format!(
            "For more information, see DOCS#{}",
            code.to_ascii_lowercase()
        );
        let (kind, kind_name, color) = match diagnostic.kind {
            DiagnosticKind::Error => (ReportKind::Error, "Error", Color::Red),
            DiagnosticKind::Warning => (ReportKind::Warning, "Warning", Color::Yellow),
        };
        let source = diagnostic.message.span().and_then(|span| {
            context
                .sources
                .get(&span.source_id)
                .map(|source| (span, source))
        });
        let Some((span, source)) = source else {
            eprintln!("[{code}] {kind_name}: {title}");
            eprintln!("  {description}");
            eprintln!("  Help: {help}");
            continue;
        };

        let path = source.path.display().to_string();
        let range = source.content[..span.start].chars().count()
            ..source.content[..span.end].chars().count();

        if let Err(error) = Report::build(kind, (path.clone(), range.clone()))
            .with_code(code)
            .with_message(title)
            .with_label(
                Label::new((path, range))
                    .with_message(description)
                    .with_color(color),
            )
            .with_help(help)
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
        .stage(
            LintPass::default()
                .lint(EmptyWord)
                .lint(UnusedImport::default()),
        )
        .stage(CollectSymbolsStage)
        .stage(LowerStage);

    print_errors(&pipeline.context, &pipeline.diagnostics.items);

    pipeline.stage(IRInterpreter::default());
}
