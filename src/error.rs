use crate::common::Span;

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub errors: Vec<CompileError>,
    pub fatal: bool,
}

impl Diagnostics {
    pub fn emit_fatal(&mut self, error: CompileError) {
        self.fatal = true;
        self.errors.push(error);
    }

    pub fn append(&mut self, mut other: Self) {
        self.fatal |= other.fatal;
        self.errors.append(&mut other.errors);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompileError {
    Unknown {
        label: String,
    },
    /// Entrypoint file not found
    FileNotFound {
        path: String,
    },
    /// Import error
    ImportError {
        path: String,
        span: Span,
    },
    UnexpectedToken {
        span: Span,
    },
    ParseError {
        label: Vec<String>,
        span: Span,
    },
    InvalidAttribute {
        name: String,
        span: Span,
    },
    SymbolAlreadyExists {
        name: String,
        span: Span,
    },
    SymbolNotFound {
        name: String,
        span: Span,
    },
    InvalidSymbol {
        name: String,
        span: Span,
    },
    RecursiveStruct {
        name: String,
        span: Span,
    },
    InvalidFieldIndex {
        name: String,
        index: usize,
        field_count: usize,
        span: Span,
    },
    InvalidStack {
        label: String,
        span: Span,
        expected_stack: Vec<String>,
        actual_stack: Vec<String>,
    },
}

impl Default for CompileError {
    fn default() -> Self {
        Self::Unknown {
            label: "Unknown".to_string(),
        }
    }
}

impl CompileError {
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::ImportError { span, .. }
            | Self::UnexpectedToken { span }
            | Self::ParseError { span, .. }
            | Self::InvalidAttribute { span, .. }
            | Self::SymbolAlreadyExists { span, .. }
            | Self::SymbolNotFound { span, .. }
            | Self::InvalidSymbol { span, .. }
            | Self::RecursiveStruct { span, .. }
            | Self::InvalidFieldIndex { span, .. }
            | Self::InvalidStack { span, .. } => Some(*span),
            Self::Unknown { .. } | Self::FileNotFound { .. } => None,
        }
    }
}
