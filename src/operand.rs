use std::convert::TryInto;
use std::fmt;

use crate::ones_complement;
use crate::DecodeError;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    RegisterDirect(u8),
    Indexed((u8, i16)),
    RegisterIndirect(u8),
    RegisterIndirectAutoIncrement(u8),
    Symbolic(i16),
    Immediate(i16),
    Absolute(u16),
    Constant(i8),
}

impl Operand {
    pub fn len(&self) -> usize {
        match self {
            Operand::RegisterDirect(_) => 0,
            Operand::Indexed(_) => 2,
            Operand::RegisterIndirect(_) => 0,
            Operand::RegisterIndirectAutoIncrement(_) => 0,
            Operand::Symbolic(_) => 2,
            Operand::Immediate(_) => 2,
            Operand::Absolute(_) => 2,
            Operand::Constant(_) => 0,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::RegisterDirect(r) => match r {
                0 => write!(f, "pc"),
                1 => write!(f, "sp"),
                2 => write!(f, "sr"),
                3 => write!(f, "cg"),
                _ => write!(f, "r{}", r),
            },
            Operand::Indexed((r, i)) => match r {
                1 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(sp)", i)
                    } else {
                        write!(f, "-{:#x}(sp)", i * -1)
                    }
                }
                3 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(cg)", i)
                    } else {
                        write!(f, "-{:#x}(cg)", i * -1)
                    }
                }
                4..=15 => {
                    if *i >= 0 {
                        write!(f, "{:#x}(r{})", i, r)
                    } else {
                        write!(f, "-{:#x}(r{})", i * -1, r)
                    }
                }
                _ => unreachable!(),
            },
            Operand::RegisterIndirect(r) => {
                if *r == 1 {
                    write!(f, "@sp")
                } else {
                    write!(f, "@r{}", r)
                }
            }
            Operand::RegisterIndirectAutoIncrement(r) => {
                if *r == 1 {
                    write!(f, "@sp+")
                } else {
                    write!(f, "@r{}+", r)
                }
            }
            // TODO: is this correct? can you know what this is without knowing what PC is?
            Operand::Symbolic(i) => {
                if *i >= 0 {
                    write!(f, "#{:#x}(pc)", i)
                } else {
                    write!(f, "#-{:#x}(pc)", i * -1)
                }
            }
            Operand::Immediate(i) => {
                if *i >= 0 {
                    write!(f, "#{:#x}", i)
                } else {
                    write!(f, "#-{:#x}", i * -1)
                }
            }
            Operand::Absolute(a) => write!(f, "&{:#x}", a),
            Operand::Constant(i) => {
                if *i >= 0 {
                    write!(f, "#{:#x}", i)
                } else {
                    write!(f, "#-{:#x}", i * -1)
                }
            }
        }
    }
}

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

pub fn parse_source(register: u8, source: u16, data: &[u8]) -> Result<(Operand, &[u8])> {
    match register {
        0 => match source {
            1 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Operand::Symbolic(second_word), remaining_data))
                }
            }
            3 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Operand::Immediate(second_word), remaining_data))
                }
            }
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        2 => match source {
            1 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word = u16::from_le_bytes(bytes.try_into().unwrap());
                    Ok((Operand::Absolute(second_word), remaining_data))
                }
            }
            2 => Ok((Operand::Constant(4), data)),
            3 => Ok((Operand::Constant(8), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        3 => match source {
            0 => Ok((Operand::Constant(0), data)),
            1 => Ok((Operand::Constant(1), data)),
            2 => Ok((Operand::Constant(2), data)),
            3 => Ok((Operand::Constant(-1), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        _ => match source {
            0 => Ok((Operand::RegisterDirect(register), data)),
            1 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Operand::Indexed((register, second_word)), remaining_data))
                }
            }
            2 => Ok((Operand::RegisterIndirect(register), data)),
            3 => Ok((Operand::RegisterIndirectAutoIncrement(register), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
    }
}

pub fn parse_destination(register: u8, source: u16, data: &[u8]) -> Result<Operand> {
    match source {
        0 => Ok(Operand::RegisterDirect(register)),
        1 => {
            if data.len() < 2 {
                Err(DecodeError::MissingDestination)
            } else {
                let (bytes, _) = data[0..2].split_at(std::mem::size_of::<u16>());
                let raw_operand = u16::from_le_bytes(bytes.try_into().unwrap());
                let index = ones_complement(raw_operand);
                match register {
                    0 => Ok(Operand::Symbolic(index)),
                    2 => Ok(Operand::Absolute(raw_operand)),
                    1 | 3..=15 => Ok(Operand::Indexed((register, index))),
                    _ => Err(DecodeError::InvalidDestination),
                }
            }
        }
        _ => Err(DecodeError::InvalidDestination),
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
    fn source_pc_immediate_negative() {
        let data = [0xfe, 0xff];
        let source = parse_source(0, 3, &data);
        assert_eq!(source, Ok((Operand::Immediate(-1), &data[2..])));
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
        assert_eq!(source, Ok((Operand::Indexed((9, -2)), &data[2..])));
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
        assert_eq!(destination, Ok(Operand::Indexed((9, -1))));
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
        assert_eq!(destination, Ok(Operand::Symbolic(-1)));
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
        assert_eq!(destination, Err(DecodeError::InvalidDestination));
    }
}
