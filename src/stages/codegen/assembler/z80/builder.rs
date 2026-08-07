use std::fmt::Display;

use super::{
    assembly::Z80Assembly,
    condition::Z80Condition,
    operand::{Z80Label, Z80Operand},
    register::Z80Register,
};

#[derive(Debug)]
pub struct Z80;

impl Z80 {
    pub fn comment(comment: impl Display) -> Z80Assembly {
        Z80Assembly::Comment(comment.to_string())
    }

    #[must_use]
    pub const fn label(label: Z80Label) -> Z80Assembly {
        Z80Assembly::Label(label)
    }

    #[must_use]
    pub const fn org(address: isize) -> Z80Assembly {
        Z80Assembly::Org(address)
    }

    #[must_use]
    pub const fn db(byte: u8) -> Z80Assembly {
        Z80Assembly::Db(byte)
    }

    #[must_use]
    pub const fn defs(size: usize) -> Z80Assembly {
        Z80Assembly::Defs(size)
    }

    pub fn ld(destination: impl Into<Z80Operand>, source: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("ld", [destination.into(), source.into()])
    }

    pub fn inc(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("inc", [operand.into()])
    }

    pub fn dec(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("dec", [operand.into()])
    }

    pub fn add(destination: impl Into<Z80Operand>, source: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("add", [destination.into(), source.into()])
    }

    pub fn sub(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("sub", [operand.into()])
    }

    pub fn sbc(destination: impl Into<Z80Operand>, source: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("sbc", [destination.into(), source.into()])
    }

    pub fn and(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("and", [operand.into()])
    }

    pub fn or(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("or", [operand.into()])
    }

    pub fn cp(operand: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("cp", [operand.into()])
    }

    #[must_use]
    pub fn push(register: Z80Register) -> Z80Assembly {
        Self::instruction("push", [register.into()])
    }

    #[must_use]
    pub fn pop(register: Z80Register) -> Z80Assembly {
        Self::instruction("pop", [register.into()])
    }

    pub fn call(target: impl Into<Z80Operand>) -> Z80Assembly {
        Self::instruction("call", [target.into()])
    }

    pub fn jp(condition: Option<Z80Condition>, target: impl Into<Z80Operand>) -> Z80Assembly {
        let mut operands = Vec::with_capacity(2);
        if let Some(condition) = condition {
            operands.push(Z80Operand::Condition(condition));
        }
        operands.push(target.into());
        Z80Assembly::Instruction {
            mnemonic: "jp",
            operands,
        }
    }

    pub fn jr(condition: Z80Condition, target: impl Into<Z80Operand>) -> Z80Assembly {
        Z80Assembly::Instruction {
            mnemonic: "jr",
            operands: vec![Z80Operand::Condition(condition), target.into()],
        }
    }

    #[must_use]
    pub fn ret() -> Z80Assembly {
        Self::instruction("ret", [])
    }

    fn instruction<const N: usize>(
        mnemonic: &'static str,
        operands: [Z80Operand; N],
    ) -> Z80Assembly {
        Z80Assembly::Instruction {
            mnemonic,
            operands: operands.into(),
        }
    }
}
