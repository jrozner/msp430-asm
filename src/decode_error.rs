/// Catch all error type that contains any error that can occur during the
/// decoding process
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodeError {
    /// Present when an instruction expects an additional source argument
    /// (after the instruction) but none is present
    MissingSource,
    /// Present when an instruction expects an additional destination argument
    /// (after the instruction) but none is present
    MissingDestination,
    /// Present when the combination of the AS (source addressing mode) field
    /// and the register are an invalid combination
    InvalidSource((u16, u8)),
    /// Present when the combination of the AD (destination addressing mode) field
    /// and the register are an invalid combination
    InvalidDestination((u16, u8)),
    /// Present when there is not instruction available to read
    MissingInstruction,
    /// Present when the opcode specified for a type 1 or type 2 instruction
    /// is invalid
    InvalidOpcode(u16),
    /// Present when the condition of a jxx instruction is invalid
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
            Self::InvalidDestination((source, register)) => {
                write!(
                    f,
                    "destination addressing mode ({}) for register ({}) is invalid",
                    source, register
                )
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
