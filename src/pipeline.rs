use crate::error::{CompileError, Diagnostics};

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

    pub fn new(output: Option<T>, diagnostics: Diagnostics) -> Self {
        Self {
            output,
            diagnostics,
        }
    }
}

pub struct PipelineBuilder<O, C> {
    prev: Option<O>,
    pub context: C,
    pub diagnostics: Diagnostics,
}

impl<O, C: Default> PipelineBuilder<O, C> {
    pub fn new(init: O) -> Self {
        Self {
            prev: Some(init),
            context: C::default(),
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

    pub fn dump(&self) -> Result<&O, &Vec<CompileError>> {
        if !self.diagnostics.errors.is_empty() {
            Err(&self.diagnostics.errors)
        } else {
            self.prev.as_ref().ok_or(&self.diagnostics.errors)
        }
    }

    pub fn finish(self) -> Result<O, Vec<CompileError>> {
        if !self.diagnostics.errors.is_empty() {
            Err(self.diagnostics.errors)
        } else {
            self.prev.ok_or_else(Vec::new)
        }
    }
}

pub trait Stage<C> {
    type Input;
    type Output;

    fn execute(&mut self, input: Self::Input, context: &mut C) -> StageResult<Self::Output>;
}
