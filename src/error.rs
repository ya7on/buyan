use crate::common::Span;

pub struct Diagnostic {
    pub errors: Vec<CompileError>,
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
