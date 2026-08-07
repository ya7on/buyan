use std::{fmt::Display, marker::PhantomData};

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

impl<A: Display> Emitter<A> {
    pub fn emit(&mut self, assembly: A) {
        self.output.push_str(&assembly.to_string());
        self.output.push('\n');
    }

    #[must_use]
    pub fn finish(self) -> String {
        self.output
    }
}
