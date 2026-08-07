use std::fmt::{self, Display, Formatter};

use super::operand::{Z80Label, Z80Operand};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Z80Assembly {
    Comment(String),
    Label(Z80Label),
    Org(isize),
    Db(u8),
    Defs(usize),
    Instruction {
        mnemonic: &'static str,
        operands: Vec<Z80Operand>,
    },
}

impl Display for Z80Assembly {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comment(comment) => write!(formatter, "; {comment}"),
            Self::Label(label) => write!(formatter, "{label}:"),
            Self::Org(address) => write!(formatter, "ORG 0x{address:x}"),
            Self::Db(byte) => write!(formatter, "\tdb {byte}"),
            Self::Defs(size) => write!(formatter, "\tdefs {size}"),
            Self::Instruction { mnemonic, operands } => {
                write!(formatter, "\t{mnemonic}")?;
                if let Some((first, rest)) = operands.split_first() {
                    write!(formatter, " {first}")?;
                    for operand in rest {
                        write!(formatter, ", {operand}")?;
                    }
                }
                Ok(())
            }
        }
    }
}
