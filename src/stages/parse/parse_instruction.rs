use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra::Err,
    input::ValueInput,
    prelude::{choice, just, recursive},
    select,
};

use crate::{
    common::{DottedPath, SourceSpan, Spanned},
    stages::parse::{
        ast::{ASTInstruction, ASTLiteral},
        lexer::TokenKind,
        parse_stack_effect::stack_effect_parser,
    },
};

#[must_use]
pub fn instruction_parser<'src, I>()
-> impl Parser<'src, I, ASTInstruction, Err<Rich<'src, TokenKind, SourceSpan>>>
where
    I: ValueInput<'src, Token = TokenKind, Span = SourceSpan>,
{
    recursive(|instr| {
        let literal = select! {
            TokenKind::LiteralBool(value) => ASTInstruction::Literal(ASTLiteral::Bool(value)),
            TokenKind::LiteralString(value) => ASTInstruction::Literal(ASTLiteral::String(value)),
            TokenKind::LiteralU8(value) => ASTInstruction::Literal(ASTLiteral::U8(value)),
            TokenKind::LiteralU16(value) => ASTInstruction::Literal(ASTLiteral::U16(value)),
        };

        let path = select! {
            TokenKind::Ident(name) => name,
        }
        .separated_by(just(TokenKind::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(DottedPath);

        let pack = just(TokenKind::GreaterThan)
            .ignore_then(path.clone())
            .map(ASTInstruction::Pack);

        let unpack = path
            .clone()
            .then_ignore(just(TokenKind::GreaterThan))
            .map(ASTInstruction::Unpack);

        let call_segment = select! {
            TokenKind::Ident(name) => name,
            TokenKind::LiteralNumber(number) => number.to_string(),
        };
        let call = select! {
            TokenKind::Ident(name) => name,
        }
        .then(
            just(TokenKind::Dot)
                .ignore_then(call_segment)
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(|(first, rest)| {
            ASTInstruction::Call(DottedPath(std::iter::once(first).chain(rest).collect()))
        });

        let lambda = stack_effect_parser()
            .delimited_by(just(TokenKind::Pipe), just(TokenKind::Pipe))
            .map_with(|stack_effect, extra| {
                let span: SourceSpan = extra.span();
                Spanned::new(stack_effect, span)
            })
            .then(
                instr
                    .clone()
                    .map_with(|instr, extra| {
                        let span: SourceSpan = extra.span();
                        Spanned::new(instr, span)
                    })
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace)),
            )
            .map(|(stack_effect, body)| ASTInstruction::Lambda { stack_effect, body });

        choice((lambda, literal, pack, unpack, call))
    })
}
