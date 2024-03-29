use std::fmt;

use crate::DecodeError;
use crate::Result;

/// Represents a source or destination operand. This represents all
/// addressing mode represented by AS/AD with their corresponding register
/// pairs. In msp430 the valid destination operands are a subset of the
/// source operands. Due to cases in the implementation where it is necessary
/// to sometimes use a source as a destination (br emulated instruction) or
/// compare a source and a destination rather than create separate types for
/// source and destination they share one. The enforcement that a valid
/// destination is specified, as all operands are valid for source, is left
/// to the implementation of the decoding logic or assembling logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    /// The operand is stored in the register
    RegisterDirect(u8),
    /// The operand is stored at the offset of the address specified in the
    /// register.
    ///
    /// This requires an additional word
    Indexed((u8, i16)),
    /// The operand is stored at the address that is in the register
    ///
    /// This requires an additional word
    RegisterIndirect(u8),
    /// The operand is stored at the address that is in the register and the
    /// register is autoincremented by one word
    RegisterIndirectAutoIncrement(u8),
    /// The operand is the value of the following word relative to PC
    ///
    /// This requires an additional word
    Symbolic(i16),
    /// The operand is the immediate value following the instruction word
    ///
    /// This requires an additional word
    Immediate(u16),
    /// The operand is stored at the address specified by the immediate value
    /// after the instruction word
    ///
    /// This requires an additional word
    Absolute(u16),
    /// The operand is a constant value specified by the combination of
    /// register (SR or CG) and the addressing mode
    Constant(i8),
}

impl Operand {
    pub fn size(&self) -> usize {
        match self {
            Self::RegisterDirect(_) => 0,
            Self::Indexed(_) => 2,
            Self::RegisterIndirect(_) => 0,
            Self::RegisterIndirectAutoIncrement(_) => 0,
            Self::Symbolic(_) => 2,
            Self::Immediate(_) => 2,
            Self::Absolute(_) => 2,
            Self::Constant(_) => 0,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegisterDirect(r) => match r {
                0 => write!(f, "pc"),
                1 => write!(f, "sp"),
                2 => write!(f, "sr"),
                3 => write!(f, "cg"),
                _ => write!(f, "r{}", r),
            },
            Self::Indexed((r, i)) => match r {
                1 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(sp)", i)
                    } else {
                        write!(f, "-{:#x}(sp)", -i)
                    }
                }
                3 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(cg)", i)
                    } else {
                        write!(f, "-{:#x}(cg)", -i)
                    }
                }
                4..=15 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(r{})", i, r)
                    } else {
                        write!(f, "-{:#x}(r{})", -i, r)
                    }
                }
                _ => unreachable!(),
            },
            Self::RegisterIndirect(r) => {
                if *r == 1 {
                    write!(f, "@sp")
                } else {
                    write!(f, "@r{}", r)
                }
            }
            Self::RegisterIndirectAutoIncrement(r) => {
                if *r == 1 {
                    write!(f, "@sp+")
                } else {
                    write!(f, "@r{}+", r)
                }
            }
            Self::Symbolic(i) => {
                if *i >= 0 {
                    write!(f, "#{:#x}(pc)", i)
                } else {
                    write!(f, "#-{:#x}(pc)", -i)
                }
            }
            Self::Immediate(i) => {
                if *i & 0x8000 == 0 {
                    write!(f, "#{:#x}", i)
                } else {
                    write!(f, "#-{:#x}", *i as i16)
                }
            }
            Self::Absolute(a) => write!(f, "&{:#x}", a),
            Self::Constant(i) => {
                if *i >= 0 {
                    write!(f, "#{:#x}", i)
                } else {
                    write!(f, "#-{:#x}", -i)
                }
            }
        }
    }
}

/// Specifies whether the operand (source or destination) will be used as a
/// byte or a word.
///
/// The operand itself is always stored as a word for alignment reasons
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperandWidth {
    Byte,
    Word,
}

impl From<u8> for OperandWidth {
    fn from(val: u8) -> Self {
        match val {
            0 => OperandWidth::Word,
            1 => OperandWidth::Byte,
            _ => unreachable!(),
        }
    }
}

/// Parses a source operand from an input stream. This is only used for AS
/// modes where the source operand is stored as an additional word of data.
/// Otherwise the source operand can be fully decoded from just reading the
/// the instruction word
pub fn parse_source(register: u8, source: u16, data: &[u8]) -> Result<(Operand, &[u8])> {
    match source {
        0 => match register {
            3 => Ok((Operand::Constant(0), data)),
            0..=2 | 4..=15 => Ok((Operand::RegisterDirect(register), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        1 => match register {
            0 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word = i16::from_le_bytes(bytes.try_into().unwrap());
                    Ok((Operand::Symbolic(second_word), remaining_data))
                }
            }
            2 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word = u16::from_le_bytes(bytes.try_into().unwrap());
                    Ok((Operand::Absolute(second_word), remaining_data))
                }
            }
            3 => Ok((Operand::Constant(1), data)),
            1 | 4..=15 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word = i16::from_le_bytes(bytes.try_into().unwrap());
                    Ok((Operand::Indexed((register, second_word)), remaining_data))
                }
            }
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        2 => match register {
            2 => Ok((Operand::Constant(4), data)),
            3 => Ok((Operand::Constant(2), data)),
            0..=1 | 4..=15 => Ok((Operand::RegisterIndirect(register), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        3 => match register {
            0 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word = u16::from_le_bytes(bytes.try_into().unwrap());
                    Ok((Operand::Immediate(second_word), remaining_data))
                }
            }
            2 => Ok((Operand::Constant(8), data)),
            3 => Ok((Operand::Constant(-1), data)),
            1 | 4..=15 => Ok((Operand::RegisterIndirectAutoIncrement(register), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        _ => Err(DecodeError::InvalidSource((source, register))),
    }
}

/// Parses a destination operand from an input stream. This is only used for
/// AD modes where the destination operand is stored as an additional word
/// of data. Otherwise the destination operand can be fully decoded from just
/// reading the the instruction word
pub fn parse_destination(register: u8, source: u16, data: &[u8]) -> Result<Operand> {
    match source {
        0 => Ok(Operand::RegisterDirect(register)),
        1 => {
            if data.len() < 2 {
                Err(DecodeError::MissingDestination)
            } else {
                let (bytes, _) = data[0..2].split_at(std::mem::size_of::<u16>());
                let raw_operand = u16::from_le_bytes(bytes.try_into().unwrap());
                let index = raw_operand;
                match register {
                    0 => Ok(Operand::Symbolic(index as i16)),
                    2 => Ok(Operand::Absolute(raw_operand)),
                    1 | 3..=15 => Ok(Operand::Indexed((register, index as i16))),
                    _ => Err(DecodeError::InvalidDestination((source, register))),
                }
            }
        }
        _ => Err(DecodeError::InvalidDestination((source, register))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_pc_symbolic() {
        let data = [0x2, 0x0];
        let source = parse_source(0, 1, &data);
        assert_eq!(source, Ok((Operand::Symbolic(2), &data[2..])));
    }

    #[test]
    fn source_pc_symbolic_missing_data() {
        let data = [];
        let source = parse_source(0, 1, &data);
        assert_eq!(source, Err(DecodeError::MissingSource))
    }

    #[test]
    fn source_pc_immediate() {
        let data = [0x2, 0x0];
        let source = parse_source(0, 3, &data);
        assert_eq!(source, Ok((Operand::Immediate(2), &data[2..])));
    }

    #[test]
    fn source_pc_immediate_high_bit() {
        let data = [0xfe, 0xff];
        let source = parse_source(0, 3, &data);
        assert_eq!(source, Ok((Operand::Immediate(65534), &data[2..])));
    }

    #[test]
    fn source_pc_immediate_missing_data() {
        let data = [];
        let source = parse_source(0, 3, &data);
        assert_eq!(source, Err(DecodeError::MissingSource))
    }

    #[test]
    fn source_pc_invalid_source() {
        let data = [0xfe, 0xff];
        let source = parse_source(0, 5, &data);
        assert_eq!(source, Err(DecodeError::InvalidSource((5, 0))));
    }

    #[test]
    fn source_sr_absolute() {
        let data = [0x2, 0x0];
        let source = parse_source(2, 1, &data);
        assert_eq!(source, Ok((Operand::Absolute(2), &data[2..])));
    }

    #[test]
    fn source_sr_absolute_missing_data() {
        let data = [];
        let source = parse_source(2, 1, &data);
        assert_eq!(source, Err(DecodeError::MissingSource));
    }

    #[test]
    fn source_sr_constant_four() {
        let data = [];
        let source = parse_source(2, 2, &data);
        assert_eq!(source, Ok((Operand::Constant(4), &data[..])));
    }

    #[test]
    fn source_sr_constant_eight() {
        let data = [];
        let source = parse_source(2, 3, &data);
        assert_eq!(source, Ok((Operand::Constant(8), &data[..])));
    }

    #[test]
    fn source_sr_invalid_source() {
        let data = [];
        let source = parse_source(2, 4, &data);
        assert_eq!(source, Err(DecodeError::InvalidSource((4, 2))));
    }

    #[test]
    fn source_cg_zero() {
        let data = [];
        let source = parse_source(3, 0, &data);
        assert_eq!(source, Ok((Operand::Constant(0), &data[..])));
    }

    #[test]
    fn source_cg_one() {
        let data = [];
        let source = parse_source(3, 1, &data);
        assert_eq!(source, Ok((Operand::Constant(1), &data[..])));
    }

    #[test]
    fn source_cg_two() {
        let data = [];
        let source = parse_source(3, 2, &data);
        assert_eq!(source, Ok((Operand::Constant(2), &data[..])));
    }

    #[test]
    fn source_cg_negative_one() {
        let data = [];
        let source = parse_source(3, 3, &data);
        assert_eq!(source, Ok((Operand::Constant(-1), &data[..])));
    }

    #[test]
    fn source_cg_invalid_source() {
        let data = [];
        let source = parse_source(3, 4, &data);
        assert_eq!(source, Err(DecodeError::InvalidSource((4, 3))));
    }

    #[test]
    fn source_gp_register_direct() {
        let data = [];
        let source = parse_source(9, 0, &data);
        assert_eq!(source, Ok((Operand::RegisterDirect(9), &data[..])));
    }

    #[test]
    fn source_gp_register_indexed() {
        let data = [0x2, 0x0];
        let source = parse_source(9, 1, &data);
        assert_eq!(source, Ok((Operand::Indexed((9, 2)), &data[2..])));
    }

    #[test]
    fn source_gp_register_indexed_negative() {
        let data = [0xfd, 0xff];
        let source = parse_source(9, 1, &data);
        assert_eq!(source, Ok((Operand::Indexed((9, -3)), &data[2..])));
    }

    #[test]
    fn source_gp_register_indirect() {
        let data = [];
        let source = parse_source(9, 2, &data);
        assert_eq!(source, Ok((Operand::RegisterIndirect(9), &data[..])));
    }

    #[test]
    fn source_gp_register_indirect_auto_increment() {
        let data = [];
        let source = parse_source(9, 3, &data);
        assert_eq!(
            source,
            Ok((Operand::RegisterIndirectAutoIncrement(9), &data[..]))
        );
    }

    #[test]
    fn source_gp_invalid_source() {
        let data = [];
        let source = parse_source(9, 4, &data);
        assert_eq!(source, Err(DecodeError::InvalidSource((4, 9))));
    }

    #[test]
    fn destination_register_direct() {
        let data = [];
        let destination = parse_destination(9, 0, &data);
        assert_eq!(destination, Ok(Operand::RegisterDirect(9)));
    }

    #[test]
    fn destination_register_indexed() {
        let data = [0x2, 0x0];
        let destination = parse_destination(9, 1, &data);
        assert_eq!(destination, Ok(Operand::Indexed((9, 2))));
    }

    #[test]
    fn destination_register_indexed_negative() {
        let data = [0xfe, 0xff];
        let destination = parse_destination(9, 1, &data);
        assert_eq!(destination, Ok(Operand::Indexed((9, -2))));
    }

    #[test]
    fn destination_register_symbolic() {
        let data = [0x2, 0x0];
        let destination = parse_destination(0, 1, &data);
        assert_eq!(destination, Ok(Operand::Symbolic(2)));
    }

    #[test]
    fn destination_register_symbolic_negative() {
        let data = [0xfe, 0xff];
        let destination = parse_destination(0, 1, &data);
        assert_eq!(destination, Ok(Operand::Symbolic(-2)));
    }

    #[test]
    fn destination_register_absolute() {
        let data = [0x2, 0x0];
        let destination = parse_destination(2, 1, &data);
        assert_eq!(destination, Ok(Operand::Absolute(2)));
    }

    #[test]
    fn destination_invalid_source() {
        let data = [];
        let destination = parse_destination(9, 3, &data);
        assert_eq!(destination, Err(DecodeError::InvalidDestination((3, 9))));
    }
}
