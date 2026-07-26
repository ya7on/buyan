use chumsky::{
    Parser,
    error::RichPattern,
    input::{Input, Stream},
    span::Span,
};

use crate::{
    common::{SourceId, SourceSpan},
    error::DiagnosticMessage,
    stages::parse::{ast::ASTModule, lexer::TokenKind, parse_module::module_parser},
};

pub struct ParserInput {
    pub tokens: Vec<(TokenKind, SourceSpan)>,
    pub source_id: SourceId,
    pub content_len: usize,
}

pub struct ParseResult {
    pub ast: ASTModule,
}

pub fn parse(input: ParserInput) -> Result<ParseResult, Vec<DiagnosticMessage>> {
    let end_span = SourceSpan::new(input.source_id, input.content_len..input.content_len);
    let token_stream =
        Stream::from_iter(input.tokens.to_owned()).map(end_span, |(t, s): (_, _)| (t, s));

    let ast = module_parser()
        .parse(token_stream)
        .into_result()
        .map_err(|errors| {
            let mut result = Vec::with_capacity(errors.len());
            for err in errors {
                let span = err.span();
                result.push(DiagnosticMessage::ParseError {
                    label: err
                        .expected()
                        .filter(|expected| matches!(expected, RichPattern::Label(_)))
                        .map(|label| format!("{label:?}"))
                        .collect(),
                    span: (*span).into(),
                });
            }
            result
        })?;

    Ok(ParseResult { ast })
}
