use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ConnectorDirection {
    Input,
    Output,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Connector {
    pub name: String,
    pub direction: ConnectorDirection,
    pub idx: usize,
    pub state: bool,
}

impl Connector {
    pub(crate) fn new(name: String, direction: ConnectorDirection, idx: usize) -> Connector {
        Connector {
            name,
            direction,
            idx,
            state: false,
        }
    }
}

impl Display for Connector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.name)
    }
}

impl PartialEq for Connector {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.direction == other.direction
    }
}
