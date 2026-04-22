use crate::error::CompileError;

pub struct PipelineBuilder<O, C> {
    prev: Result<O, Vec<CompileError>>,
    context: C,
}

impl<O, C: Default> PipelineBuilder<O, C> {
    pub fn new(init: O) -> Self {
        Self {
            prev: Ok(init),
            context: C::default(),
        }
    }

    pub fn stage_initialized<T>(mut self, mut stage: T) -> PipelineBuilder<T::Output, C>
    where
        T: Stage<C, Input = O>,
    {
        let prev = match self.prev {
            Ok(prev) => prev,
            Err(errors) => {
                return PipelineBuilder {
                    prev: Err(errors),
                    context: self.context,
                };
            }
        };
        let result = stage.execute(prev, &mut self.context);
        match result {
            Ok(output) => PipelineBuilder {
                prev: Ok(output),
                context: self.context,
            },
            Err(error) => PipelineBuilder {
                prev: Err(error),
                context: self.context,
            },
        }
    }

    pub fn stage<T>(self) -> PipelineBuilder<T::Output, C>
    where
        T: Stage<C, Input = O>,
    {
        let stage = T::default();
        self.stage_initialized(stage)
    }

    pub fn dump(&self) -> Result<&O, &Vec<CompileError>> {
        self.prev.as_ref()
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn finish(self) -> Result<O, Vec<CompileError>> {
        self.prev
    }
}

pub trait Stage<C>: Default {
    type Input;
    type Output;

    fn execute(
        &mut self,
        input: Self::Input,
        context: &mut C,
    ) -> Result<Self::Output, Vec<CompileError>>;
}
