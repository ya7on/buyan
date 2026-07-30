use chumsky::{
    IterParser, Parser, error::Rich, extra::Err, input::ValueInput, prelude::just, select,
};

use crate::{
    common::{SourceSpan, Spanned},
    stages::parse::{ast::ASTStruct, lexer::TokenKind, parse_stack_effect::stack_item_parser},
};

#[must_use]
pub fn struct_parser<'src, I>()
-> impl Parser<'src, I, ASTStruct, Err<Rich<'src, TokenKind, SourceSpan>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SourceSpan>,
{
    let name = select! { TokenKind::Ident(name) => name }.map_with(|name, extra| {
        let span: SourceSpan = extra.span();
        Spanned::new(name, span)
    });

    just(TokenKind::KeywordStruct)
        .ignore_then(name.labelled("Struct name was expected"))
        .then(
            stack_item_parser()
                .map_with(|field, extra| {
                    let span: SourceSpan = extra.span();
                    Spanned::new(field, span)
                })
                .separated_by(just(TokenKind::Comma))
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(
                    just(TokenKind::LeftParenthesis),
                    just(TokenKind::RightParenthesis),
                ),
        )
        .then_ignore(just(TokenKind::Semicolon).labelled("; was expected after struct declaration"))
        .map(|(name, fields)| ASTStruct { name, fields })
}
