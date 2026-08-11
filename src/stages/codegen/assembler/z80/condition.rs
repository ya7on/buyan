use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Z80Condition {
    C,
    Nc,
    Nz,
    Z,
}

impl Display for Z80Condition {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::C => "c",
            Self::Nc => "nc",
            Self::Nz => "nz",
            Self::Z => "z",
        };
        formatter.write_str(name)
    }
}
