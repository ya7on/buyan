use chumsky::{
    IterParser, Parser, error::Rich, extra::Err, input::ValueInput, prelude::just, select,
    span::SimpleSpan,
};

use crate::{
    common::{DottedPath, Spanned},
    stages::parse::{ast::ASTModule, lexer::TokenKind, parse_word::word_parser},
};

#[must_use]
pub fn module_parser<'src, I>() -> impl Parser<'src, I, ASTModule, Err<Rich<'src, TokenKind>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SimpleSpan>,
{
    let import_parser = just(TokenKind::KeywordImport)
        .ignore_then(
            select! { TokenKind::Ident(name) => name }
                .separated_by(just(TokenKind::Dot))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(TokenKind::Semicolon))
        .map_with(|name, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(DottedPath(name), span)
        });

    let module_name_parser = just(TokenKind::KeywordModule)
        .ignore_then(
            select! { TokenKind::Ident(name) => name }
                .separated_by(just(TokenKind::Dot))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(TokenKind::Semicolon).labelled("; was expected after module name"))
        .map_with(|name, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(DottedPath(name), span)
        });

    import_parser
        .repeated()
        .collect::<Vec<_>>()
        .then(module_name_parser)
        .then(
            word_parser()
                .map_with(|func, extra| {
                    let span: SimpleSpan = extra.span();
                    Spanned::new(func, span)
                })
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(|((imports, name), words)| ASTModule {
            imports,
            name,
            words,
        })
}
