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
            DecodeError::MissingSource => {
                write!(f, "source operand is missing")
            }
            DecodeError::MissingDestination => {
                write!(f, "destination operand is missing")
            }
            DecodeError::InvalidSource((source, register)) => {
                write!(
                    f,
                    "source addressing mode ({}) for register ({}) is invalid",
                    source, register
                )
            }
            DecodeError::InvalidDestination => {
                write!(f, "destination addressing mode is invalid")
            }
            DecodeError::MissingInstruction => {
                write!(f, "not enough data to decode instruction")
            }
            DecodeError::InvalidOpcode(opcode) => {
                write!(f, "invalid opcode {}", opcode)
            }
            DecodeError::InvalidJumpCondition(condition) => {
                write!(f, "invalid jump condition {}", condition)
            }
        }
    }
}

impl std::error::Error for DecodeError {}
