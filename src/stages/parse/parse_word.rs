use chumsky::{
    IterParser, Parser, error::Rich, extra::Err, input::ValueInput, prelude::just, select,
};

use crate::{
    common::{SourceSpan, Spanned},
    stages::parse::{
        ast::{ASTWord, ASTWordVar},
        lexer::TokenKind,
        parse_attribute::attributes_parser,
        parse_instruction::instruction_parser,
        parse_stack_effect::stack_effect_parser,
    },
};

#[must_use]
pub fn word_parser<'src, I>()
-> impl Parser<'src, I, ASTWord, Err<Rich<'src, TokenKind, SourceSpan>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SourceSpan>,
{
    let word_name = select! {
        TokenKind::Ident(name) => name,
    }
    .map_with(|name, extra| {
        let span: SourceSpan = extra.span();
        Spanned::new(name, span)
    });

    let stack_var = just(TokenKind::Ellipsis)
        .ignore_then(select! { TokenKind::Ident(name) => name })
        .map_with(|name, extra| {
            let span: SourceSpan = extra.span();
            ASTWordVar::Stack {
                name: Spanned::new(name, span),
            }
        });
    let type_var = select! { TokenKind::Ident(name) => name }
        .map_with(|name, extra| {
            let span: SourceSpan = extra.span();
            Spanned::new(name, span)
        })
        .map(|name| ASTWordVar::Type { name });
    let word_vars = type_var
        .or(stack_var)
        .separated_by(just(TokenKind::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .labelled("Stack effect type vars expected here");

    attributes_parser()
        .then_ignore(just(TokenKind::KeywordDef))
        .then(word_name.labelled("Word name was expected"))
        .then(
            word_vars
                .delimited_by(just(TokenKind::LessThan), just(TokenKind::GreaterThan))
                .or_not()
                .map(Option::unwrap_or_default),
        )
        .then(
            stack_effect_parser()
                .map_with(|effect, extra| {
                    let span: SourceSpan = extra.span();
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
                    let span: SourceSpan = extra.span();
                    Spanned::new(instruction, span)
                })
                .repeated()
                .collect::<Vec<_>>(),
        )
        .then_ignore(
            just(TokenKind::KeywordEnd).labelled("Expected 'end' to close word definition"),
        )
        .map(
            |((((attributes, name), word_vars), stack_effect), body)| ASTWord {
                name,
                body,
                word_vars,
                stack_effect,
                attributes,
            },
        )
}
