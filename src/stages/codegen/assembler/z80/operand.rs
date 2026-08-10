use std::fmt::{self, Display, Formatter};

use super::{condition::Z80Condition, register::Z80Register};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Z80Immediate(pub isize);

impl Display for Z80Immediate {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Z80Label(String);

impl Z80Label {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Display for Z80Label {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Z80Operand {
    Register(Z80Register),
    Condition(Z80Condition),
    Immediate(Z80Immediate),
    Label(Z80Label),
    LabelOffset { label: Z80Label, offset: isize },
    Indirect(Z80Register),
    IndirectLabel(Z80Label),
    Indexed { base: Z80Register, offset: isize },
}

impl Z80Operand {
    #[must_use]
    pub const fn indirect(register: Z80Register) -> Self {
        Self::Indirect(register)
    }

    #[must_use]
    pub fn indirect_label(label: Z80Label) -> Self {
        Self::IndirectLabel(label)
    }

    #[must_use]
    pub const fn indexed(base: Z80Register, offset: isize) -> Self {
        Self::Indexed { base, offset }
    }

    #[must_use]
    pub const fn label_offset(label: Z80Label, offset: isize) -> Self {
        Self::LabelOffset { label, offset }
    }
}

impl From<Z80Register> for Z80Operand {
    fn from(value: Z80Register) -> Self {
        Self::Register(value)
    }
}

impl From<Z80Immediate> for Z80Operand {
    fn from(value: Z80Immediate) -> Self {
        Self::Immediate(value)
    }
}

impl From<Z80Label> for Z80Operand {
    fn from(value: Z80Label) -> Self {
        Self::Label(value)
    }
}

impl Display for Z80Operand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(register) => register.fmt(formatter),
            Self::Condition(condition) => condition.fmt(formatter),
            Self::Immediate(immediate) => immediate.fmt(formatter),
            Self::Label(label) => label.fmt(formatter),
            Self::LabelOffset { label, offset } if *offset < 0 => {
                write!(formatter, "{label}{offset}")
            }
            Self::LabelOffset { label, offset } => write!(formatter, "{label}+{offset}"),
            Self::Indirect(register) => write!(formatter, "({register})"),
            Self::IndirectLabel(label) => write!(formatter, "({label})"),
            Self::Indexed { base, offset } if *offset < 0 => write!(formatter, "({base}{offset})"),
            Self::Indexed { base, offset } => write!(formatter, "({base}+{offset})"),
        }
    }
}
