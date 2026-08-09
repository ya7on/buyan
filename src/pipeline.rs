use crate::error::{Diagnostic, DiagnosticKind, Diagnostics};

#[derive(Debug)]
pub struct StageResult<T> {
    output: Option<T>,
    diagnostics: Diagnostics,
}

impl<T> StageResult<T> {
    pub fn success(output: T) -> Self {
        Self {
            output: Some(output),
            diagnostics: Diagnostics::default(),
        }
    }

    pub const fn new(output: Option<T>, diagnostics: Diagnostics) -> Self {
        Self {
            output,
            diagnostics,
        }
    }
}

#[derive(Debug)]
pub struct PipelineBuilder<O, C> {
    prev: Option<O>,
    pub context: C,
    pub diagnostics: Diagnostics,
}

impl<O, C> PipelineBuilder<O, C> {
    pub fn new(init: O, context: C) -> Self {
        Self {
            prev: Some(init),
            context,
            diagnostics: Diagnostics::default(),
        }
    }
    pub fn stage<T>(mut self, mut stage: T) -> PipelineBuilder<T::Output, C>
    where
        T: Stage<C, Input = O>,
    {
        if self.diagnostics.fatal {
            return PipelineBuilder {
                prev: None,
                context: self.context,
                diagnostics: self.diagnostics,
            };
        }

        let Some(prev) = self.prev else {
            return PipelineBuilder {
                prev: None,
                context: self.context,
                diagnostics: self.diagnostics,
            };
        };
        let outcome = stage.execute(prev, &mut self.context);
        self.diagnostics.append(outcome.diagnostics);
        PipelineBuilder {
            prev: outcome.output,
            context: self.context,
            diagnostics: self.diagnostics,
        }
    }

    /// Returns the current pipeline output.
    ///
    /// # Errors
    /// Returns collected diagnostics when the pipeline failed or has no output.
    pub fn dump(&self) -> Result<&O, &Vec<Diagnostic>> {
        if self
            .diagnostics
            .items
            .iter()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::Error)
        {
            Err(&self.diagnostics.items)
        } else {
            self.prev.as_ref().ok_or(&self.diagnostics.items)
        }
    }
}

pub trait Stage<C> {
    type Input;
    type Output;

    fn execute(&mut self, input: Self::Input, context: &mut C) -> StageResult<Self::Output>;
}
