use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Z80Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    IX,
}

impl Display for Z80Register {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::H => "h",
            Self::L => "l",
            Self::AF => "af",
            Self::BC => "bc",
            Self::DE => "de",
            Self::HL => "hl",
            Self::IX => "ix",
        };
        formatter.write_str(name)
    }
}
