use chumsky::{
    IterParser, Parser, error::Rich, extra::Err, input::ValueInput, prelude::just, select,
    span::SimpleSpan,
};

use crate::{
    common::Spanned,
    stages::parse::{
        ast::{ASTWord, ASTWordVar},
        lexer::TokenKind,
        parse_instruction::instruction_parser,
        parse_stack_effect::stack_effect_parser,
    },
};

#[must_use]
pub fn word_parser<'src, I>() -> impl Parser<'src, I, ASTWord, Err<Rich<'src, TokenKind>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SimpleSpan>,
{
    let attributes = just(TokenKind::Hash)
        .ignore_then(
            select! {
                TokenKind::Ident(name) => name,
            }
            .delimited_by(
                just(TokenKind::LeftSquareBracket),
                just(TokenKind::RightSquareBracket),
            ),
        )
        .map_with(|name, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(name, span)
        });

    let word_name = select! {
        TokenKind::Ident(name) => name,
    }
    .map_with(|name, extra| {
        let span: SimpleSpan = extra.span();
        Spanned::new(name, span)
    });

    let stack_var = just(TokenKind::Ellipsis)
        .ignore_then(select! { TokenKind::Ident(name) => name })
        .map_with(|name, extra| {
            let span: SimpleSpan = extra.span();
            ASTWordVar::Stack {
                name: Spanned::new(name, span),
            }
        });
    let type_var = select! { TokenKind::Ident(name) => name }
        .map_with(|name, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(name, span)
        })
        .then(
            just(TokenKind::Colon)
                .ignore_then(
                    select! {
                        TokenKind::Ident(name) => name,
                    }
                    .map_with(|name, extra| {
                        let span: SimpleSpan = extra.span();
                        Spanned::new(name, span)
                    })
                    .separated_by(just(TokenKind::Plus))
                    .collect::<Vec<_>>(),
                )
                .or_not()
                .map(|traits| traits.unwrap_or_default()),
        )
        .map(|(name, traits)| ASTWordVar::Type { name, traits });
    let word_vars = type_var
        .or(stack_var)
        .separated_by(just(TokenKind::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .labelled("Stack effect type vars expected here");

    (attributes
        .repeated()
        .collect::<Vec<_>>()
        .or_not()
        .map(|attrs| attrs.unwrap_or_default()))
    .then_ignore(just(TokenKind::KeywordDef))
    .then(word_name.labelled("Word name was expected"))
    .then(
        word_vars
            .delimited_by(just(TokenKind::LessThan), just(TokenKind::GreaterThan))
            .or_not()
            .map(|type_vars| type_vars.unwrap_or_default()),
    )
    .then(
        stack_effect_parser()
            .map_with(|effect, extra| {
                let span: SimpleSpan = extra.span();
                Spanned::new(effect, span)
            })
            .labelled("Stack effect was expected")
            .delimited_by(
                just(TokenKind::LeftParenthesis),
                just(TokenKind::RightParenthesis),
            )
            .labelled("Stack effect was expected"),
    )
    .then(
        instruction_parser()
            .map_with(|instruction, extra| {
                let span: SimpleSpan = extra.span();
                Spanned::new(instruction, span)
            })
            .repeated()
            .collect::<Vec<_>>(),
    )
    .then_ignore(just(TokenKind::KeywordEnd).labelled("Expected 'end' to close word definition"))
    .map(
        |((((attributes, name), word_vars), stack_effect), body)| ASTWord {
            name,
            word_vars,
            body,
            stack_effect,
            attributes,
        },
    )
}
