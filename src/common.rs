use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    path::PathBuf,
};

use chumsky::span::SimpleSpan;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SourceId(pub usize);

pub type SourceSpan = SimpleSpan<usize, SourceId>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        Self {
            source_id: span.context,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    #[default]
    Interpreter,
    Z80UnknownCpm,
}

impl Display for CompileTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interpreter => formatter.write_str("interpreter"),
            Self::Z80UnknownCpm => formatter.write_str("z80-unknown-cpm"),
        }
    }
}

impl TryFrom<&str> for CompileTarget {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "interpreter" => Ok(Self::Interpreter),
            "z80-unknown-cpm" => Ok(Self::Z80UnknownCpm),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct CompileContext {
    pub sources: HashMap<SourceId, Source>,
    pub target: CompileTarget,
    source_ids: HashMap<PathBuf, SourceId>,
}

impl CompileContext {
    #[must_use]
    pub fn new(target: CompileTarget) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    pub fn add_source(&mut self, path: PathBuf, content: String) -> SourceId {
        if let Some(source_id) = self.source_ids.get(&path) {
            return *source_id;
        }

        let source_id = SourceId(self.sources.len());
        self.source_ids.insert(path.clone(), source_id);
        self.sources.insert(source_id, Source { path, content });
        source_id
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DottedPath(pub Vec<String>);

impl DottedPath {
    /// Appends a name to the dotted path, returning a new `DottedPath` instance.
    #[must_use]
    pub fn append(&self, name: &str) -> Self {
        Self(
            self.0
                .clone()
                .into_iter()
                .chain(Some(name.to_string()))
                .collect(),
        )
    }

    #[must_use]
    pub fn extend(&self, other: &Self) -> Self {
        Self(
            self.0
                .clone()
                .into_iter()
                .chain(other.0.iter().cloned())
                .collect(),
        )
    }

    #[must_use]
    pub fn parse(path: &str) -> Self {
        Self(path.split('.').map(ToString::to_string).collect())
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn first(&self) -> Option<&str> {
        self.0.first().map(String::as_str)
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
        Self::from(path.0.join("."))
    }
}
