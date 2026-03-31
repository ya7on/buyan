use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    path::PathBuf,
};

use chumsky::span::SimpleSpan;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<SimpleSpan> for Span {
    fn from(span: SimpleSpan) -> Self {
        (&span).into()
    }
}

impl From<&SimpleSpan> for Span {
    fn from(span: &SimpleSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Spanned<T: Debug> {
    pub value: T,
    pub span: Span,
}

impl<T: Debug> Spanned<T> {
    pub fn new<A, B>(value: A, span: B) -> Self
    where
        A: Into<T>,
        B: Into<Span>,
    {
        Self {
            value: value.into(),
            span: span.into(),
        }
    }
}

impl<T: Debug> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Default)]
pub struct CompileContext {
    pub sources: HashMap<PathBuf, String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DottedPath(pub Vec<String>);

impl DottedPath {
    /// Appends a name to the dotted path, returning a new `DottedPath` instance.
    pub fn append(&self, name: &str) -> Self {
        Self(
            self.0
                .clone()
                .into_iter()
                .chain(Some(name.to_string()))
                .collect(),
        )
    }

    pub fn extend(&self, other: &Self) -> Self {
        Self(
            self.0
                .clone()
                .into_iter()
                .chain(other.0.iter().cloned())
                .collect(),
        )
    }

    pub fn parse(path: &str) -> Self {
        Self(path.split('.').map(|s| s.to_string()).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first(&self) -> Option<&str> {
        self.0.first().map(|s| s.as_str())
    }
}

impl Display for DottedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let joined = self.0.join(".");
        write!(f, "{joined}")
    }
}

impl From<DottedPath> for PathBuf {
    fn from(path: DottedPath) -> Self {
        PathBuf::from(path.0.join("."))
    }
}
