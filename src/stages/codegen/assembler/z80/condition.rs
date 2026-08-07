use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Z80Condition {
    Nz,
}

impl Display for Z80Condition {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Nz => "nz",
        };
        formatter.write_str(name)
    }
}
