#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    MissingSource,
    MissingDestination,
    InvalidSource((u16, u8)),
    InvalidDestination,
    MissingInstruction,
    InvalidOpcode(u16),
    InvalidJumpCondition(u16),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSource => {
                write!(f, "source operand is missing")
            }
            Self::MissingDestination => {
                write!(f, "destination operand is missing")
            }
            Self::InvalidSource((source, register)) => {
                write!(
                    f,
                    "source addressing mode ({}) for register ({}) is invalid",
                    source, register
                )
            }
            Self::InvalidDestination => {
                write!(f, "destination addressing mode is invalid")
            }
            Self::MissingInstruction => {
                write!(f, "not enough data to decode instruction")
            }
            Self::InvalidOpcode(opcode) => {
                write!(f, "invalid opcode {}", opcode)
            }
            Self::InvalidJumpCondition(condition) => {
                write!(f, "invalid jump condition {}", condition)
            }
        }
    }
}

impl std::error::Error for DecodeError {}
