use chumsky::{
    Parser,
    error::RichPattern,
    input::{Input, Stream},
    span::SimpleSpan,
};

use crate::{
    error::CompileError,
    stages::parse::{ast::ASTModule, lexer::TokenKind, parse_module::module_parser},
};

pub struct ParserInput {
    pub tokens: Vec<(TokenKind, SimpleSpan)>,
}

pub struct ParseResult {
    pub ast: ASTModule,
}

pub fn parse(input: ParserInput) -> Result<ParseResult, Vec<CompileError>> {
    let token_stream = Stream::from_iter(input.tokens.to_owned())
        .map((0..input.tokens.len()).into(), |(t, s): (_, _)| (t, s));

    let ast = module_parser()
        .parse(token_stream)
        .into_result()
        .map_err(|errors| {
            let mut result = Vec::with_capacity(errors.len());
            for err in errors {
                let span = err.span();
                result.push(CompileError::ParseError {
                    label: err
                        .expected()
                        .filter(|expected| matches!(expected, RichPattern::Label(_)))
                        .map(|label| format!("{label:?}"))
                        .collect(),
                    span: span.into(),
                });
            }
            result
        })?;

    Ok(ParseResult { ast })
}
