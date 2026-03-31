use crate::{
    common::CompileContext,
    error::CompileError,
    pipeline::Stage,
    stages::parse::ast::{
        ASTInstruction, ASTLiteral, ASTModule, ASTProgram, ASTStackEffect, ASTStackEffectItem,
        ASTWord, ASTWordVar,
    },
};
use std::fmt::Write;

fn dump_stack_item(buf: &mut String, item: &ASTStackEffectItem) -> Result<(), std::fmt::Error> {
    match item {
        ASTStackEffectItem::Symbol { name } => write!(buf, "{}", name)?,
        ASTStackEffectItem::StackVar { name } => write!(buf, "...{}", name)?,
        ASTStackEffectItem::Lambda { stack_effect } => {
            let mut s = String::new();
            dump_stack_effect(&mut s, stack_effect)?;
            write!(buf, "|{}|", s)?;
        }
    }
    Ok(())
}

fn dump_stack_effect(buf: &mut String, item: &ASTStackEffect) -> Result<(), std::fmt::Error> {
    let mut in_items = Vec::with_capacity(item.stack_in.len());
    for item in &item.stack_in {
        let mut s = String::new();
        dump_stack_item(&mut s, item)?;
        in_items.push(s);
    }
    write!(buf, "{}", in_items.join(", "))?;
    write!(buf, " -- ")?;
    let mut out_items = Vec::with_capacity(item.stack_out.len());
    for item in &item.stack_out {
        let mut s = String::new();
        dump_stack_item(&mut s, item)?;
        out_items.push(s);
    }
    write!(buf, "{}", out_items.join(", "))?;
    Ok(())
}

fn dump_instruction(buf: &mut String, instruction: &ASTInstruction) -> Result<(), std::fmt::Error> {
    match &instruction {
        ASTInstruction::Call(name) => write!(buf, "{name}")?,
        ASTInstruction::Literal(ASTLiteral::String(value)) => write!(buf, "{value}")?,
        ASTInstruction::Literal(ASTLiteral::U8(value)) => write!(buf, "{value}")?,
        ASTInstruction::Lambda { stack_effect, body } => {
            let mut s = String::new();
            dump_stack_effect(&mut s, stack_effect)?;
            let mut b = Vec::new();
            for instruction in body {
                let mut s = String::new();
                dump_instruction(&mut s, instruction)?;
                b.push(s);
            }
            write!(buf, "|{}| {{ {} }}", s, b.join(" "))?;
        }
    }
    Ok(())
}

fn dump_word(buf: &mut String, word: &ASTWord) -> Result<(), std::fmt::Error> {
    write!(buf, "def {}", *word.name)?;
    if !word.word_vars.is_empty() {
        write!(buf, "<")?;
        let vars = word
            .word_vars
            .iter()
            .map(|var| match var {
                ASTWordVar::Stack { name } => name.to_string(),
                ASTWordVar::Type { name, traits } => {
                    let traits = traits
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(" + ");
                    vec![name.to_string(), traits]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(": ")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(buf, "{}", vars)?;
        write!(buf, ">")?;
    }
    write!(buf, "(")?;
    dump_stack_effect(buf, &word.stack_effect)?;
    writeln!(buf, ")")?;

    let mut instructions = Vec::with_capacity(word.body.len());
    for instruction in &word.body {
        let mut s = String::new();
        dump_instruction(&mut s, instruction)?;
        instructions.push(s);
    }
    if !instructions.is_empty() {
        writeln!(buf, "\t{}", instructions.join("\n\t"))?;
    }

    writeln!(buf, "end\n")?;
    Ok(())
}

fn dump_module(buf: &mut String, module: &ASTModule) -> Result<(), std::fmt::Error> {
    writeln!(buf, "module \"{}\"\n", *module.name)?;

    for word in &module.words {
        dump_word(buf, word)?;
    }

    Ok(())
}

#[derive(Default)]
pub struct DumpAst;

impl Stage<CompileContext> for DumpAst {
    type Input = ASTProgram;
    type Output = ASTProgram;

    fn execute(
        &mut self,
        input: Self::Input,
        _: &mut CompileContext,
    ) -> Result<Self::Output, Vec<CompileError>> {
        let mut result = String::new();
        for module in &input.modules {
            dump_module(&mut result, module).map_err(|e| {
                vec![CompileError::Unknown {
                    label: e.to_string(),
                }]
            })?;
        }
        println!("{}", result);

        Ok(input)
    }
}
