use chumsky::{
    IterParser, Parser, error::Rich, extra::Err, input::ValueInput, prelude::just, select,
    span::SimpleSpan,
};

use crate::{
    common::{DottedPath, Spanned},
    stages::parse::{ast::ASTStruct, lexer::TokenKind},
};

#[must_use]
pub fn struct_parser<'src, I>() -> impl Parser<'src, I, ASTStruct, Err<Rich<'src, TokenKind>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SimpleSpan>,
{
    let name = select! { TokenKind::Ident(name) => name }.map_with(|name, extra| {
        let span: SimpleSpan = extra.span();
        Spanned::new(name, span)
    });

    let field = select! { TokenKind::Ident(name) => name }
        .separated_by(just(TokenKind::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map_with(|path, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(DottedPath(path), span)
        });

    just(TokenKind::KeywordStruct)
        .ignore_then(name.labelled("Struct name was expected"))
        .then(
            field
                .separated_by(just(TokenKind::Comma))
                .collect::<Vec<_>>()
                .delimited_by(
                    just(TokenKind::LeftParenthesis),
                    just(TokenKind::RightParenthesis),
                ),
        )
        .then_ignore(just(TokenKind::Semicolon).labelled("; was expected after struct declaration"))
        .map(|(name, fields)| ASTStruct { name, fields })
}
