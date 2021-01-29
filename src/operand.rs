use std::convert::TryInto;

use crate::ones_complement;
use crate::DecodeError;
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    RegisterDirect(u8),
    Indexed((u8, i16)),
    RegisterIndirect(u8),
    IndirectAutoIncrement(u8),
    Symbolic(i16),
    Immediate(i16),
    Absolute(u16),
    Constant(i8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    RegisterDirect(u8),
    Indexed((u8, i16)),
}

pub fn parse_source(register: u8, source: u16, data: &[u8]) -> Result<(Source, &[u8])> {
    match register {
        0 => match source {
            1 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Source::Symbolic(second_word), remaining_data))
                }
            }
            3 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Source::Immediate(second_word), remaining_data))
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
                    Ok((Source::Absolute(second_word), remaining_data))
                }
            }
            2 => Ok((Source::Constant(4), data)),
            3 => Ok((Source::Constant(8), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        3 => match source {
            0 => Ok((Source::Constant(0), data)),
            1 => Ok((Source::Constant(1), data)),
            2 => Ok((Source::Constant(2), data)),
            3 => Ok((Source::Constant(-1), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
        _ => match source {
            0 => Ok((Source::RegisterDirect(register), data)),
            1 => {
                if data.len() < 2 {
                    Err(DecodeError::MissingSource)
                } else {
                    let (bytes, remaining_data) = data.split_at(std::mem::size_of::<u16>());
                    let second_word =
                        ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                    Ok((Source::Indexed((register, second_word)), remaining_data))
                }
            }
            2 => Ok((Source::RegisterIndirect(register), data)),
            3 => Ok((Source::IndirectAutoIncrement(register), data)),
            _ => Err(DecodeError::InvalidSource((source, register))),
        },
    }
}

pub fn parse_destination(register: u8, source: u16, data: &[u8]) -> Result<Destination> {
    match source {
        0 => Ok(Destination::RegisterDirect(register)),
        1 => {
            if data.len() < 2 {
                Err(DecodeError::MissingDestination)
            } else {
                let (bytes, _) = data[0..2].split_at(std::mem::size_of::<u16>());
                let index = ones_complement(u16::from_le_bytes(bytes.try_into().unwrap()));
                Ok(Destination::Indexed((register, index)))
            }
        }
        _ => Err(DecodeError::InvalidDestination),
    }
}

mod tests {
    use super::*;

    #[test]
    fn source_pc_symbolic() {
        let data = [0x2, 0x0];
        let source = parse_source(0, 1, &data);
        assert_eq!(source, Ok((Source::Symbolic(2), &data[2..])));
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
        assert_eq!(source, Ok((Source::Immediate(2), &data[2..])));
    }

    #[test]
    fn source_pc_immediate_negative() {
        let data = [0xfe, 0xff];
        let source = parse_source(0, 3, &data);
        assert_eq!(source, Ok((Source::Immediate(-1), &data[2..])));
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
        assert_eq!(source, Ok((Source::Absolute(2), &data[2..])));
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
        assert_eq!(source, Ok((Source::Constant(4), &data[..])));
    }

    #[test]
    fn source_sr_constant_eight() {
        let data = [];
        let source = parse_source(2, 3, &data);
        assert_eq!(source, Ok((Source::Constant(8), &data[..])));
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
        assert_eq!(source, Ok((Source::Constant(0), &data[..])));
    }

    #[test]
    fn source_cg_one() {
        let data = [];
        let source = parse_source(3, 1, &data);
        assert_eq!(source, Ok((Source::Constant(1), &data[..])));
    }

    #[test]
    fn source_cg_two() {
        let data = [];
        let source = parse_source(3, 2, &data);
        assert_eq!(source, Ok((Source::Constant(2), &data[..])));
    }

    #[test]
    fn source_cg_negative_one() {
        let data = [];
        let source = parse_source(3, 3, &data);
        assert_eq!(source, Ok((Source::Constant(-1), &data[..])));
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
        assert_eq!(source, Ok((Source::RegisterDirect(9), &data[..])));
    }

    #[test]
    fn source_gp_register_indexed() {
        let data = [0x2, 0x0];
        let source = parse_source(9, 1, &data);
        assert_eq!(source, Ok((Source::Indexed((9, 2)), &data[2..])));
    }

    #[test]
    fn source_gp_register_indexed_negative() {
        let data = [0xfd, 0xff];
        let source = parse_source(9, 1, &data);
        assert_eq!(source, Ok((Source::Indexed((9, -2)), &data[2..])));
    }

    #[test]
    fn source_gp_register_indirect() {
        let data = [];
        let source = parse_source(9, 2, &data);
        assert_eq!(source, Ok((Source::RegisterIndirect(9), &data[..])));
    }

    #[test]
    fn source_gp_register_indirect_auto_increment() {
        let data = [];
        let source = parse_source(9, 3, &data);
        assert_eq!(source, Ok((Source::IndirectAutoIncrement(9), &data[..])));
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
        assert_eq!(destination, Ok(Destination::RegisterDirect(9)));
    }

    #[test]
    fn destination_register_indexed() {
        let data = [0x2, 0x0];
        let destination = parse_destination(9, 1, &data);
        assert_eq!(destination, Ok(Destination::Indexed((9, 2))));
    }

    #[test]
    fn destination_invalid_source() {
        let data = [];
        let destination = parse_destination(9, 3, &data);
        assert_eq!(destination, Err(DecodeError::InvalidDestination));
    }
}
