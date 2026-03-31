use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra::Err,
    input::ValueInput,
    prelude::{just, recursive},
    select,
    span::SimpleSpan,
};

use crate::{
    common::Spanned,
    stages::parse::{
        ast::{ASTStackEffect, ASTStackEffectItem},
        lexer::TokenKind,
    },
};

#[must_use]
pub fn stack_item_parser<'src, I>()
-> impl Parser<'src, I, ASTStackEffectItem, Err<Rich<'src, TokenKind>>> + Clone
where
    I: ValueInput<'src, Token = TokenKind, Span = SimpleSpan>,
{
    recursive(|ty| {
        let stack_var = just(TokenKind::Ellipsis).ignore_then(
            select! { TokenKind::Ident(name) => name }
                .map(|name| ASTStackEffectItem::StackVar { name }),
        );
        let typed = select! { TokenKind::Ident(name) => name }
            .map(|name| ASTStackEffectItem::Symbol { name });
        let lambda = ty
            .clone()
            .map_with(|instr, extra| {
                let span: SimpleSpan = extra.span();
                Spanned::new(instr, span)
            })
            .separated_by(just(TokenKind::Comma))
            .collect::<Vec<_>>()
            .then_ignore(just(TokenKind::MinusMinus))
            .then(
                ty.map_with(|instr, extra| {
                    let span: SimpleSpan = extra.span();
                    Spanned::new(instr, span)
                })
                .separated_by(just(TokenKind::Comma))
                .collect::<Vec<_>>(),
            )
            .delimited_by(just(TokenKind::Pipe), just(TokenKind::Pipe))
            .map_with(|(stack_in, stack_out), extra| {
                let span: SimpleSpan = extra.span();
                ASTStackEffectItem::Lambda {
                    stack_effect: Spanned::new(
                        ASTStackEffect {
                            stack_in,
                            stack_out,
                        },
                        span,
                    ),
                }
            });

        stack_var.or(typed).or(lambda)
    })
}

#[must_use]
pub fn stack_effect_parser<'src, I>()
-> impl Parser<'src, I, ASTStackEffect, Err<Rich<'src, TokenKind>>> + Clone
where
    I: ValueInput<'src, Token = TokenKind, Span = SimpleSpan>,
{
    let stack_in = stack_item_parser()
        .map_with(|value, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(value, span)
        })
        .separated_by(just(TokenKind::Comma))
        .collect::<Vec<_>>();
    let stack_out = stack_item_parser()
        .map_with(|value, extra| {
            let span: SimpleSpan = extra.span();
            Spanned::new(value, span)
        })
        .separated_by(just(TokenKind::Comma))
        .collect::<Vec<_>>();

    stack_in
        .then_ignore(just(TokenKind::MinusMinus))
        .then(stack_out)
        .map(|(stack_in, stack_out)| ASTStackEffect {
            stack_in,
            stack_out,
        })
}
