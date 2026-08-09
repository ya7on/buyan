use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra::Err,
    input::ValueInput,
    prelude::{just, select},
};

use crate::{
    common::{SourceSpan, Spanned},
    stages::parse::{ast::ASTAttribute, lexer::TokenKind},
};

#[must_use]
pub fn attributes_parser<'src, I>()
-> impl Parser<'src, I, Vec<Spanned<ASTAttribute>>, Err<Rich<'src, TokenKind, SourceSpan>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SourceSpan>,
{
    let attribute = select! {
        TokenKind::Ident(name) => name,
    }
    .then(
        just(TokenKind::Equal)
            .ignore_then(select! {
                TokenKind::LiteralString(value) => value,
            })
            .or_not(),
    )
    .map(|(name, value)| ASTAttribute { name, value })
    .delimited_by(
        just(TokenKind::LeftSquareBracket),
        just(TokenKind::RightSquareBracket),
    );

    just(TokenKind::Hash)
        .ignore_then(attribute)
        .map_with(|attribute, extra| {
            let span: SourceSpan = extra.span();
            Spanned::new(attribute, span)
        })
        .repeated()
        .collect()
}
