use std::convert::TryInto;

pub mod decode_error;
pub mod instruction;
pub mod jxx;
pub mod operand;
pub mod single_operand;
pub mod two_operand;

use decode_error::DecodeError;
use instruction::Instruction;
use jxx::*;
use operand::{parse_destination, parse_source, Source};
use single_operand::*;
use two_operand::*;

const RRC_OPCODE: u16 = 0;
const SWPB_OPCODE: u16 = 1;
const RRA_OPCODE: u16 = 2;
const SXT_OPCODE: u16 = 3;
const PUSH_OPCODE: u16 = 4;
const CALL_OPCODE: u16 = 5;
const RETI_OPCODE: u16 = 6;

const MOV_OPCODE: u16 = 4;
const ADD_OPCODE: u16 = 5;
const ADDC_OPCODE: u16 = 6;
const SUBC_OPCODE: u16 = 7;
const SUB_OPCODE: u16 = 8;
const CMP_OPCODE: u16 = 9;
const DADD_OPCODE: u16 = 10;
const BIT_OPCODE: u16 = 11;
const BIC_OPCODE: u16 = 12;
const BIS_OPCODE: u16 = 13;
const XOR_OPCODE: u16 = 14;
const AND_OPCODE: u16 = 15;

const SINGLE_OPERAND_REGISTER_MASK: u16 = 0b1111;

const SINGLE_OPERAND_OPCODE_MASK: u16 = 0b0000_0011_1000_0000;

const SINGLE_OPERAND_SOURCE_MASK: u16 = 0b11_0000;

const SINGLE_OPERAND_WIDTH_MASK: u16 = 0b100_0000;

const INST_TYPE_MASK: u16 = 0b1110_0000_0000_0000;

const SINGLE_OPERAND_INSTRUCTION: u16 = 0b0000_0000_0000_0000;

/// JMP_MASK masks off the high three bits to check whether the pattern 001
/// is present. This describes a JMP instruction
const JMP_INSTRUCTION: u16 = 0b0010_0000_0000_0000;

/// JMP_CONDITION_MASK masks off the three bits used to denote the Jxx condition
const JMP_CONDITION_MASK: u16 = 0b0001_1100_0000_0000;

/// JMP_OFFSET masks off the lower 10 bits used to represent the offset.
/// This can be a negative offset and it represented as such in one's
/// compliment
const JMP_OFFSET: u16 = 0b0000001111111111;

const TWO_OPERAND_OPCODE_MASK: u16 = 0b1111_0000_0000_0000;
const TWO_OPERAND_SOURCE_MASK: u16 = 0b1111_0000_0000;
const TWO_OPERAND_AD_MASK: u16 = 0b1000_0000;
const TWO_OPERAND_WIDTH: u16 = 0b100_0000;
const TWO_OPERAND_AS: u16 = 0b11_0000;
const TWO_OPERAND_DESTINATION: u16 = 0b1111;

pub type Result<T> = std::result::Result<T, DecodeError>;

pub fn decode(data: &[u8], addr: usize) -> Result<Instruction> {
    if data.len() < (addr + 2) {
        return Err(DecodeError::MissingInstruction);
    }

    let (int_bytes, remaining_data) = data[addr..].split_at(std::mem::size_of::<u16>());
    let first_word = u16::from_le_bytes(int_bytes.try_into().unwrap());

    let inst_type = first_word & INST_TYPE_MASK;
    match inst_type {
        SINGLE_OPERAND_INSTRUCTION => {
            let opcode = (SINGLE_OPERAND_OPCODE_MASK & first_word) >> 7;
            let register = (SINGLE_OPERAND_REGISTER_MASK & first_word) as u8;
            let source_addressing = (SINGLE_OPERAND_SOURCE_MASK & first_word) >> 4;
            let operand_width = ((SINGLE_OPERAND_WIDTH_MASK & first_word) >> 6) as u8;

            // RETI is a special condition where the source addressing,
            // operand, and operand width are are fixed
            if opcode == RETI_OPCODE {
                return Ok(Instruction::Reti(Reti::new()));
            }

            let (source, _) = operand::parse_source(register, source_addressing, remaining_data)?;

            match opcode {
                RRC_OPCODE => Ok(Instruction::Rrc(Rrc::new(source, operand_width))),
                SWPB_OPCODE => Ok(Instruction::Swpb(Swpb::new(source))),
                RRA_OPCODE => Ok(Instruction::Rra(Rra::new(source, operand_width))),
                SXT_OPCODE => Ok(Instruction::Sxt(Sxt::new(source))),
                PUSH_OPCODE => Ok(Instruction::Push(Push::new(source, operand_width))),
                CALL_OPCODE => Ok(Instruction::Call(Call::new(source))),
                _ => Err(DecodeError::InvalidOpcode(opcode)),
            }
        }
        JMP_INSTRUCTION => {
            let condition = (first_word & JMP_CONDITION_MASK) >> 10;
            let offset = jxx_fix_offset(first_word & JMP_OFFSET);
            // TODO: we may be able to simplify this by using C style
            // enums and just convert from the condition to the value
            // after checking that the condition is [0, 7)
            match condition {
                0 => Ok(Instruction::Jnz(Jnz::new(offset))),
                1 => Ok(Instruction::Jz(Jz::new(offset))),
                2 => Ok(Instruction::Jlo(Jlo::new(offset))),
                3 => Ok(Instruction::Jc(Jc::new(offset))),
                4 => Ok(Instruction::Jn(Jn::new(offset))),
                5 => Ok(Instruction::Jge(Jge::new(offset))),
                6 => Ok(Instruction::Jl(Jl::new(offset))),
                7 => Ok(Instruction::Jmp(Jmp::new(offset))),
                _ => Err(DecodeError::InvalidJumpCondition(condition)),
            }
        }
        _ => {
            // The opcode is the first four bits for this type of
            // instruction so there isn't a simple mask we can check.
            // If it doesn't match a single operand or jmp instuction
            // we'll fall through to here and attempt to match a two
            // operand. If it doesn't match any we'll return None
            let opcode = (first_word & TWO_OPERAND_OPCODE_MASK) >> 12;
            let source_register = ((first_word & TWO_OPERAND_SOURCE_MASK) >> 8) as u8;
            let ad = (first_word & TWO_OPERAND_AD_MASK) >> 7;
            let operand_width = ((first_word & TWO_OPERAND_WIDTH) >> 6) as u8;
            let source_addressing = (first_word & TWO_OPERAND_AS) >> 4;
            let destination_register = (first_word & TWO_OPERAND_DESTINATION) as u8;

            // if source has an additional word it is encoded before the destination
            let (source, remaining_data) =
                parse_source(source_register, source_addressing, remaining_data)?;

            let destination = parse_destination(destination_register, ad, remaining_data)?;

            match opcode {
                MOV_OPCODE => Ok(Instruction::Mov(Mov::new(
                    source,
                    operand_width,
                    destination,
                ))),
                ADD_OPCODE => Ok(Instruction::Add(Add::new(
                    source,
                    operand_width,
                    destination,
                ))),
                ADDC_OPCODE => Ok(Instruction::Addc(Addc::new(
                    source,
                    operand_width,
                    destination,
                ))),
                SUBC_OPCODE => Ok(Instruction::Subc(Subc::new(
                    source,
                    operand_width,
                    destination,
                ))),
                SUB_OPCODE => Ok(Instruction::Sub(Sub::new(
                    source,
                    operand_width,
                    destination,
                ))),
                CMP_OPCODE => Ok(Instruction::Cmp(Cmp::new(
                    source,
                    operand_width,
                    destination,
                ))),
                DADD_OPCODE => Ok(Instruction::Dadd(Dadd::new(
                    source,
                    operand_width,
                    destination,
                ))),
                BIT_OPCODE => Ok(Instruction::Bit(Bit::new(
                    source,
                    operand_width,
                    destination,
                ))),
                BIC_OPCODE => Ok(Instruction::Bic(Bic::new(
                    source,
                    operand_width,
                    destination,
                ))),
                BIS_OPCODE => Ok(Instruction::Bis(Bis::new(
                    source,
                    operand_width,
                    destination,
                ))),
                XOR_OPCODE => Ok(Instruction::Xor(Xor::new(
                    source,
                    operand_width,
                    destination,
                ))),
                AND_OPCODE => Ok(Instruction::And(And::new(
                    source,
                    operand_width,
                    destination,
                ))),
                _ => Err(DecodeError::InvalidOpcode(opcode)),
            }
        }
    }
}

// TODO does it make sense to create a trait for from u16 via one's complement?
// TODO write tests for this
fn ones_complement(val: u16) -> i16 {
    if 0b1000_0000_0000_0000 & val > 0 {
        -1 * !val as i16
    } else {
        val as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_data() {
        let data = [];
        assert_eq!(decode(&data, 0), Err(DecodeError::MissingInstruction));
    }

    #[test]
    fn jnz() {
        let data = [0x00, 0x20];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jnz(Jnz::new(0))));
    }

    #[test]
    fn negative_jnz() {
        let data = [0xf9, 0x23];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jnz(Jnz::new(-6))));
    }

    #[test]
    fn jz() {
        let data = [0x00, 0x24];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jz(Jz::new(0))));
    }

    #[test]
    fn jlo() {
        let data = [0x00, 0x28];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jlo(Jlo::new(0))));
    }

    #[test]
    fn jlc() {
        let data = [0x00, 0x2c];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jc(Jc::new(0))));
    }

    #[test]
    fn jn() {
        let data = [0x00, 0x30];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jn(Jn::new(0))));
    }

    #[test]
    fn jge() {
        let data = [0x00, 0x34];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jge(Jge::new(0))));
    }

    #[test]
    fn jl() {
        let data = [0x00, 0x38];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jl(Jl::new(0))));
    }

    #[test]
    fn jmp() {
        let data = [0x00, 0x3c];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Jmp(Jmp::new(0))));
    }

    #[test]
    fn rrc_w_register_direct() {
        let data = [0x09, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::RegisterDirect(9), 0)))
        )
    }

    #[test]
    fn rrc_b_register_direct() {
        let data = [0x49, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::RegisterDirect(9), 1)))
        );
    }

    #[test]
    fn rrc_w_indexed_positive() {
        let data = [0x19, 0x10, 0x4, 0x0];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::Indexed((9, 4)), 0)))
        );
    }

    #[test]
    fn rrc_w_indexed_negative() {
        let data = [0x19, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::Indexed((9, -4)), 0)))
        );
    }

    #[test]
    fn rrc_b_indexed_positive() {
        let data = [0x59, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::Indexed((9, 4)), 1)))
        );
    }

    #[test]
    fn rrc_b_indexed_negative() {
        let data = [0x59, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::Indexed((9, -4)), 1)))
        );
    }

    #[test]
    fn rrc_w_register_indirect() {
        let data = [0x29, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::RegisterIndirect(9), 0)))
        );
    }

    #[test]
    fn rrc_b_register_indirect() {
        let data = [0x69, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(Source::RegisterIndirect(9), 1)))
        );
    }

    #[test]
    fn rrc_w_register_indirect_autoincrement() {
        let data = [0x39, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(
                Source::IndirectAutoIncrement(9),
                0
            )))
        );
    }

    #[test]
    fn rrc_b_register_indirect_autoincrement() {
        let data = [0x79, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rrc(Rrc::new(
                Source::IndirectAutoIncrement(9),
                1
            )))
        );
    }

    #[test]
    fn swpb_register_direct() {
        let data = [0x89, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Swpb(Swpb::new(Source::RegisterDirect(9))))
        );
    }

    #[test]
    fn swpb_register_indexed_positive() {
        let data = [0x99, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Swpb(Swpb::new(Source::Indexed((9, 4)))))
        );
    }

    #[test]
    fn swpb_register_indexed_negative() {
        let data = [0x99, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Swpb(Swpb::new(Source::Indexed((9, -4)))))
        );
    }

    #[test]
    fn swpb_register_indirect() {
        let data = [0xa9, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Swpb(Swpb::new(Source::RegisterIndirect(9))))
        );
    }

    #[test]
    fn swpb_register_indirect_autoincrement() {
        let data = [0xb9, 0x10];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Swpb(Swpb::new(Source::IndirectAutoIncrement(
                9
            ))))
        );
    }

    #[test]
    fn rra_w_register_direct() {
        let data = [0x09, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::RegisterDirect(9), 0)))
        );
    }

    #[test]
    fn rra_b_register_direct() {
        let data = [0x49, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::RegisterDirect(9), 1)))
        );
    }

    #[test]
    fn rra_w_indexed_positive() {
        let data = [0x19, 0x11, 0x4, 0x0];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::Indexed((9, 4)), 0)))
        );
    }

    #[test]
    fn rra_w_indexed_negative() {
        let data = [0x19, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::Indexed((9, -4)), 0)))
        );
    }

    #[test]
    fn rra_b_indexed_positive() {
        let data = [0x59, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::Indexed((9, 4)), 1)))
        );
    }

    #[test]
    fn rra_b_indexed_negative() {
        let data = [0x59, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::Indexed((9, -4)), 1)))
        );
    }

    #[test]
    fn rra_w_register_indirect() {
        let data = [0x29, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::RegisterIndirect(9), 0)))
        );
    }

    #[test]
    fn rra_b_register_indirect() {
        let data = [0x69, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(Source::RegisterIndirect(9), 1)))
        );
    }

    #[test]
    fn rra_w_register_indirect_autoincrement() {
        let data = [0x39, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(
                Source::IndirectAutoIncrement(9),
                0
            )))
        );
    }

    #[test]
    fn rra_b_register_indirect_autoincrement() {
        let data = [0x79, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Rra(Rra::new(
                Source::IndirectAutoIncrement(9),
                1
            )))
        );
    }

    #[test]
    fn sxt_register_direct() {
        let data = [0x89, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Sxt(Sxt::new(Source::RegisterDirect(9))))
        );
    }

    #[test]
    fn sxt_register_indexed_positive() {
        let data = [0x99, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Sxt(Sxt::new(Source::Indexed((9, 4)))))
        );
    }

    #[test]
    fn sxt_register_indexed_negative() {
        let data = [0x99, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Sxt(Sxt::new(Source::Indexed((9, -4)))))
        );
    }

    #[test]
    fn sxt_register_indirect() {
        let data = [0xa9, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Sxt(Sxt::new(Source::RegisterIndirect(9))))
        );
    }

    #[test]
    fn sxt_register_indirect_autoincrement() {
        let data = [0xb9, 0x11];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Sxt(Sxt::new(Source::IndirectAutoIncrement(9))))
        );
    }

    #[test]
    fn push_w_register_direct() {
        let data = [0x09, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::RegisterDirect(9), 0)))
        );
    }

    #[test]
    fn push_b_register_direct() {
        let data = [0x49, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::RegisterDirect(9), 1)))
        );
    }

    #[test]
    fn push_w_indexed_positive() {
        let data = [0x19, 0x12, 0x4, 0x0];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Indexed((9, 4)), 0)))
        );
    }

    #[test]
    fn push_w_indexed_negative() {
        let data = [0x19, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Indexed((9, -4)), 0)))
        );
    }

    #[test]
    fn push_b_indexed_positive() {
        let data = [0x59, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Indexed((9, 4)), 1)))
        );
    }

    #[test]
    fn push_b_indexed_negative() {
        let data = [0x59, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Indexed((9, -4)), 1))),
        );
    }

    #[test]
    fn push_w_register_indirect() {
        let data = [0x29, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::RegisterIndirect(9), 0)))
        );
    }

    #[test]
    fn push_b_register_indirect() {
        let data = [0x69, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::RegisterIndirect(9), 1)))
        );
    }

    #[test]
    fn push_w_register_indirect_autoincrement() {
        let data = [0x39, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(
                Source::IndirectAutoIncrement(9),
                0
            )))
        );
    }

    #[test]
    fn push_b_register_indirect_autoincrement() {
        let data = [0x79, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(
                Source::IndirectAutoIncrement(9),
                1
            )))
        );
    }

    #[test]
    fn push_const_sr_one() {
        let data = [0x12, 0x12, 0x0, 0x44];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Absolute(0x4400), 0)))
        );
    }

    #[test]
    fn push_const_sr_two() {
        let data = [0x22, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(4), 0)))
        );
    }

    #[test]
    fn push_const_sr_three() {
        let data = [0x32, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(8), 0)))
        );
    }

    #[test]
    fn push_const_cg_zero() {
        let data = [0x03, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(0), 0)))
        );
    }

    #[test]
    fn push_const_cg_one() {
        let data = [0x13, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(1), 0)))
        );
    }

    #[test]
    fn push_const_cg_two() {
        let data = [0x23, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(2), 0)))
        );
    }

    #[test]
    fn push_const_cg_three() {
        let data = [0x33, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Push(Push::new(Source::Constant(-1), 0)))
        );
    }

    #[test]
    fn call_register_direct() {
        let data = [0x89, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Call(Call::new(Source::RegisterDirect(9))))
        );
    }

    #[test]
    fn call_register_indexed_positive() {
        let data = [0x99, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Call(Call::new(Source::Indexed((9, 4)))))
        );
    }

    #[test]
    fn call_register_indexed_negative() {
        let data = [0x99, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Call(Call::new(Source::Indexed((9, -4)))))
        );
    }

    #[test]
    fn call_register_indirect() {
        let data = [0xa9, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Call(Call::new(Source::RegisterIndirect(9))))
        );
    }

    #[test]
    fn call_register_indirect_autoincrement() {
        let data = [0xb9, 0x12];
        let inst = decode(&data, 0);
        assert_eq!(
            inst,
            Ok(Instruction::Call(Call::new(Source::IndirectAutoIncrement(
                9
            ))))
        );
    }

    #[test]
    fn call_pc_symbolic() {
        let data = [0x90, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Call(Call::new(Source::Symbolic(2)))));
    }

    #[test]
    fn call_pc_immediate() {
        let data = [0xb0, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Call(Call::new(Source::Immediate(2)))));
    }

    #[test]
    fn reti() {
        let data = [0x00, 0x13];
        let inst = decode(&data, 0);
        assert_eq!(inst, Ok(Instruction::Reti(Reti::new())));
    }
}
