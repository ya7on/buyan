use crate::common::Span;

pub struct Diagnostic {
    pub errors: Vec<CompileError>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompileError {
    Unknown { label: String },
    FileNotFound { path: String },
    ImportError { path: String, span: Span },
    UnexpectedToken { span: Span },
    ParseError { label: Vec<String>, span: Span },
    InvalidAttribute { name: String, span: Span },
    SymbolAlreadyExists { name: String, span: Span },
    SymbolNotFound { name: String, span: Span },
    InvalidSymbol { name: String, span: Span },
    InvalidStack { span: Span },
}

impl Default for CompileError {
    fn default() -> Self {
        Self::Unknown {
            label: "Unknown".to_string(),
        }
    }
}
