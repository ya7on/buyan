use std::marker::PhantomData;

pub trait Assembly {
    fn write_to(self, output: &mut String);
}

/// Collects assembly source one line at a time.
#[derive(Debug)]
pub struct Emitter<A> {
    output: String,
    assembly: PhantomData<A>,
}

impl<A> Default for Emitter<A> {
    fn default() -> Self {
        Self {
            output: String::new(),
            assembly: PhantomData,
        }
    }
}

impl<A: Assembly> Emitter<A> {
    pub fn emit(&mut self, assembly: A) {
        assembly.write_to(&mut self.output);
        self.output.push('\n');
    }

    #[must_use]
    pub fn finish(self) -> String {
        self.output
    }
}
