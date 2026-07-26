pub mod empty_word;
pub mod unused_import;

use crate::{
    common::{CompileContext, Spanned},
    error::Diagnostics,
    pipeline::{Stage, StageResult},
    stages::semantic::{
        context::HIRContext,
        hir::{HIRInstruction, HIRModule, HIRProgram, HIRStruct, HIRWord},
    },
};

pub trait Lint {
    fn check_program(
        &mut self,
        _ctx: &HIRContext,
        _program: &HIRProgram,
        _diagnostics: &mut Diagnostics,
    ) {
    }

    fn check_module(
        &mut self,
        _ctx: &HIRContext,
        _module: &HIRModule,
        _diagnostics: &mut Diagnostics,
    ) {
    }

    fn check_struct(
        &mut self,
        _ctx: &HIRContext,
        _hir_struct: &HIRStruct,
        _diagnostics: &mut Diagnostics,
    ) {
    }

    fn check_word(&mut self, _ctx: &HIRContext, _word: &HIRWord, _diagnostics: &mut Diagnostics) {}

    fn check_instruction(
        &mut self,
        _ctx: &HIRContext,
        _instruction: &Spanned<HIRInstruction>,
        _diagnostics: &mut Diagnostics,
    ) {
    }

    fn finish(&mut self, _ctx: &HIRContext, _program: &HIRProgram, _diagnostics: &mut Diagnostics) {
    }
}

#[derive(Default)]
pub struct LintPass {
    lints: Vec<Box<dyn Lint>>,
}

impl LintPass {
    pub fn lint(mut self, lint: impl Lint + 'static) -> Self {
        self.lints.push(Box::new(lint));
        self
    }

    fn check_instruction(
        lints: &mut [Box<dyn Lint>],
        hir_ctx: &HIRContext,
        instruction: &Spanned<HIRInstruction>,
        diagnostics: &mut Diagnostics,
    ) {
        for lint in &mut *lints {
            lint.check_instruction(hir_ctx, instruction, diagnostics);
        }

        if let HIRInstruction::Lambda { body, .. } = &instruction.value {
            for instruction in body {
                Self::check_instruction(lints, hir_ctx, instruction, diagnostics);
            }
        }
    }
}

impl Stage<CompileContext> for LintPass {
    type Input = (HIRContext, HIRProgram);
    type Output = (HIRContext, HIRProgram);

    fn execute(
        &mut self,
        (hir_ctx, hir_program): Self::Input,
        _: &mut CompileContext,
    ) -> StageResult<Self::Output> {
        let mut diagnostics = Diagnostics::default();

        for lint in &mut self.lints {
            lint.check_program(&hir_ctx, &hir_program, &mut diagnostics);
        }
        for module in &hir_program.modules {
            for lint in &mut self.lints {
                lint.check_module(&hir_ctx, module, &mut diagnostics);
            }
            for item in &module.structs {
                for lint in &mut self.lints {
                    lint.check_struct(&hir_ctx, item, &mut diagnostics);
                }
            }
            for word in &module.words {
                for lint in &mut self.lints {
                    lint.check_word(&hir_ctx, word, &mut diagnostics);
                }
                for instruction in &word.body {
                    Self::check_instruction(
                        &mut self.lints,
                        &hir_ctx,
                        instruction,
                        &mut diagnostics,
                    );
                }
            }
        }
        for lint in &mut self.lints {
            lint.finish(&hir_ctx, &hir_program, &mut diagnostics);
        }

        StageResult::new(Some((hir_ctx, hir_program)), diagnostics)
    }
}
