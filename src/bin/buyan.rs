use std::{io::Write, path::PathBuf, process::ExitCode};

use ariadne::{Color, Label, Report, ReportKind};
use buyan::{
    common::{CompileContext, CompileTarget},
    error::{Diagnostic, DiagnosticKind},
    fs::RealFileSystem,
    lints::{LintPass, empty_word::EmptyWord, unused_import::UnusedImport},
    pipeline::PipelineBuilder,
    stages::{
        codegen::z80_cpm::Z80CpmCodegenStage,
        interpreter::executor::IRInterpreter,
        lower::{collect::CollectSymbolsStage, monomorphize::MonomorphizeStage, stage::LowerStage},
        parse::stage::ParseStage,
        semantic::{collect_hir::CollectHIRStage, collect_names::CollectNamesStage},
    },
};
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Target {
    Interpreter,
    #[value(name = "z80-unknown-cpm")]
    Z80UnknownCpm,
}

impl Target {
    const fn compile_target(self) -> CompileTarget {
        match self {
            Self::Interpreter => CompileTarget::Interpreter,
            Self::Z80UnknownCpm => CompileTarget::Z80UnknownCpm,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,

    #[arg(long, value_enum, default_value_t = Target::Interpreter)]
    target: Target,
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
            "For more information, see https://ya7on.github.io/buyan/codes.html#{}",
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

        let mut report = Report::build(kind, (path.clone(), range.clone()))
            .with_code(code)
            .with_message(title)
            .with_label(
                Label::new((path, range))
                    .with_message(description)
                    .with_color(color),
            );

        for (additional_span, message) in diagnostic.message.additional_labels() {
            let Some(additional_source) = context.sources.get(&additional_span.source_id) else {
                continue;
            };
            let additional_path = additional_source.path.display().to_string();
            let additional_range = additional_source.content[..additional_span.start]
                .chars()
                .count()
                ..additional_source.content[..additional_span.end]
                    .chars()
                    .count();
            report = report.with_label(
                Label::new((additional_path, additional_range))
                    .with_message(message)
                    .with_color(Color::Cyan),
            );
        }

        if let Err(error) = report.with_help(help).finish().eprint(&mut cache) {
            eprintln!("failed to print diagnostic: {error}");
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    let pipeline =
        PipelineBuilder::new(args.path, CompileContext::new(args.target.compile_target()))
            .stage(ParseStage::<RealFileSystem>::default())
            // .stage(DumpAst)
            .stage(CollectNamesStage)
            .stage(CollectHIRStage)
            .stage(
                LintPass::default()
                    .lint(EmptyWord)
                    .lint(UnusedImport::default()),
            )
            .stage(CollectSymbolsStage)
            .stage(MonomorphizeStage)
            .stage(LowerStage);

    print_errors(&pipeline.context, &pipeline.diagnostics.items);

    let diagnostic_count = pipeline.diagnostics.items.len();
    let diagnostics = match args.target {
        Target::Interpreter => {
            let pipeline = pipeline.stage(IRInterpreter::default());
            print_errors(
                &pipeline.context,
                &pipeline.diagnostics.items[diagnostic_count..],
            );

            pipeline.diagnostics.items
        }
        Target::Z80UnknownCpm => {
            let pipeline = pipeline.stage(Z80CpmCodegenStage);
            print_errors(
                &pipeline.context,
                &pipeline.diagnostics.items[diagnostic_count..],
            );

            if let Ok(source) = pipeline.dump() {
                let mut stdout = std::io::stdout().lock();
                if let Err(error) = stdout.write_all(source.as_bytes()) {
                    eprintln!("failed to write assembly source: {error}");
                }
            }

            pipeline.diagnostics.items
        }
    };

    for diagnostic in diagnostics {
        if diagnostic.kind == DiagnosticKind::Error {
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
