use crate::common::Span;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub code: u16,
    pub message: DiagnosticMessage,
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
    pub fatal: bool,
}

impl Diagnostics {
    pub fn emit_fatal(&mut self, message: DiagnosticMessage) {
        self.fatal = true;
        self.items.push(Diagnostic {
            kind: DiagnosticKind::Error,
            code: message.code(),
            message,
        });
    }

    pub fn emit_warning(&mut self, message: DiagnosticMessage) {
        self.items.push(Diagnostic {
            kind: DiagnosticKind::Warning,
            code: message.code(),
            message,
        });
    }

    pub fn append(&mut self, mut other: Self) {
        self.fatal |= other.fatal;
        self.items.append(&mut other.items);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DiagnosticMessage {
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
        additional_spans: Vec<Span>,
        expected_stack: Vec<String>,
        actual_stack: Vec<String>,
    },
    EmptyWord {
        name: String,
        span: Span,
    },
    UnusedImport {
        name: String,
        span: Span,
    },
}

impl Default for DiagnosticMessage {
    fn default() -> Self {
        Self::Unknown {
            label: "Unknown".to_string(),
        }
    }
}

impl DiagnosticMessage {
    pub fn code(&self) -> u16 {
        match self {
            Self::Unknown { .. } => 1,
            Self::FileNotFound { .. } => 2,
            Self::ImportError { .. } => 3,
            Self::UnexpectedToken { .. } => 4,
            Self::ParseError { .. } => 5,
            Self::InvalidAttribute { .. } => 6,
            Self::SymbolAlreadyExists { .. } => 7,
            Self::SymbolNotFound { .. } => 8,
            Self::InvalidSymbol { .. } => 9,
            Self::RecursiveStruct { .. } => 10,
            Self::InvalidFieldIndex { .. } => 11,
            Self::InvalidStack { .. } => 12,
            Self::EmptyWord { .. } => 13,
            Self::UnusedImport { .. } => 14,
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::EmptyWord { span, .. }
            | Self::UnusedImport { span, .. }
            | Self::ImportError { span, .. }
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

    pub fn additional_labels(&self) -> Vec<(Span, String)> {
        match self {
            Self::InvalidStack {
                additional_spans,
                actual_stack,
                ..
            } => additional_spans
                .iter()
                .map(|span| {
                    (
                        *span,
                        format!("final stack is [{}]", actual_stack.join(", ")),
                    )
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Unknown { .. } => "Unknown Error",
            Self::FileNotFound { .. } => "File Not Found",
            Self::ImportError { .. } => "Import Error",
            Self::UnexpectedToken { .. } => "Unexpected Token",
            Self::ParseError { .. } => "Parse Error",
            Self::InvalidAttribute { .. } => "Invalid Attribute",
            Self::SymbolAlreadyExists { .. } => "Symbol Already Exists",
            Self::SymbolNotFound { .. } => "Symbol Not Found",
            Self::InvalidSymbol { .. } => "Invalid Symbol",
            Self::RecursiveStruct { .. } => "Recursive Struct",
            Self::InvalidFieldIndex { .. } => "Invalid Field Index",
            Self::InvalidStack { .. } => "Invalid Stack",
            Self::EmptyWord { .. } => "Empty Word",
            Self::UnusedImport { .. } => "Unused Import",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Unknown { label } => label.clone(),
            Self::FileNotFound { path } => format!("file '{path}' could not be found"),
            Self::ImportError { path, .. } => {
                format!("module '{path}' could not be imported")
            }
            Self::UnexpectedToken { .. } => "this token is not valid here".to_string(),
            Self::ParseError { label, .. } if label.is_empty() => {
                "the syntax is not valid here".to_string()
            }
            Self::ParseError { label, .. } => format!("expected {}", label.join(" or ")),
            Self::InvalidAttribute { name, .. } => {
                format!("attribute '{name}' is not supported")
            }
            Self::SymbolAlreadyExists { name, .. } => {
                format!("symbol '{name}' is already defined")
            }
            Self::SymbolNotFound { name, .. } => {
                format!("could not find symbol '{name}'")
            }
            Self::InvalidSymbol { name, .. } => {
                format!("symbol '{name}' cannot be used here")
            }
            Self::RecursiveStruct { name, .. } => {
                format!("struct '{name}' contains itself recursively")
            }
            Self::InvalidFieldIndex {
                name,
                index,
                field_count,
                ..
            } => format!(
                "field index {index} is out of bounds for '{name}', which has {field_count} fields"
            ),
            Self::InvalidStack {
                label,
                expected_stack,
                actual_stack,
                ..
            } => format!(
                "{label}; expected stack [{}], found [{}]",
                expected_stack.join(", "),
                actual_stack.join(", ")
            ),
            Self::EmptyWord { name, .. } => {
                format!("word '{name}' has an empty body")
            }
            Self::UnusedImport { name, .. } => {
                format!("import '{name}' is never used")
            }
        }
    }
}
