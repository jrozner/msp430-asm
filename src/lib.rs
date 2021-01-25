use std::convert::TryInto;

pub mod instruction;
pub mod jxx;
pub mod operand;
pub mod single_operand;
pub mod two_operand;

use instruction::Instruction;
use jxx::*;
use operand::{parse_destination, parse_source, Destination, Source};
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

fn decode(data: &[u8], addr: usize) -> Option<Instruction> {
    if data.len() < (addr + 2) {
        return None;
    }

    let (int_bytes, remaining_data) = data[addr..].split_at(std::mem::size_of::<u16>());
    // TODO: do we need to worry about the unwrap failing here?
    let first_word = u16::from_le_bytes(int_bytes.try_into().unwrap());

    let inst_type = first_word & INST_TYPE_MASK;
    match inst_type {
        SINGLE_OPERAND_INSTRUCTION => {
            let opcode = (SINGLE_OPERAND_OPCODE_MASK & first_word) >> 7;
            let register = (SINGLE_OPERAND_REGISTER_MASK & first_word) as u8;
            let source_addressing = (SINGLE_OPERAND_SOURCE_MASK & first_word) >> 4;
            let operand_width = ((SINGLE_OPERAND_WIDTH_MASK & first_word) >> 6) as u8;

            let (source, _) = operand::parse_source(register, source_addressing, remaining_data);

            match opcode {
                RRC_OPCODE => Some(Instruction::Rrc(Rrc::new(source.unwrap(), operand_width))),
                SWPB_OPCODE => Some(Instruction::Swpb(Swpb::new(source.unwrap()))),
                RRA_OPCODE => Some(Instruction::Rra(Rra::new(source.unwrap(), operand_width))),
                SXT_OPCODE => Some(Instruction::Sxt(Sxt::new(source.unwrap()))),
                PUSH_OPCODE => Some(Instruction::Push(Push::new(source.unwrap(), operand_width))),
                CALL_OPCODE => Some(Instruction::Call(Call::new(source.unwrap()))),
                RETI_OPCODE => Some(Instruction::Reti(Reti::new())),
                _ => None,
            }
        }
        JMP_INSTRUCTION => {
            let condition = (first_word & JMP_CONDITION_MASK) >> 10;
            let offset = jxx_fix_offset(first_word & JMP_OFFSET);
            // TODO: we may be able to simplify this by using C style
            // enums and just convert from the condition to the value
            // after checking that the condition is [0, 7)
            let inst = match condition {
                0 => Instruction::Jnz(Jnz::new(offset)),
                1 => Instruction::Jz(Jz::new(offset)),
                2 => Instruction::Jlo(Jlo::new(offset)),
                3 => Instruction::Jc(Jc::new(offset)),
                4 => Instruction::Jn(Jn::new(offset)),
                5 => Instruction::Jge(Jge::new(offset)),
                6 => Instruction::Jl(Jl::new(offset)),
                7 => Instruction::Jmp(Jmp::new(offset)),
                _ => unreachable!(),
            };
            Some(inst)
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
                parse_source(source_register, source_addressing, remaining_data);

            let destination = parse_destination(destination_register, ad, remaining_data);

            let inst = match opcode {
                MOV_OPCODE => {
                    Instruction::Mov(Mov::new(source.unwrap(), operand_width, destination))
                }
                ADD_OPCODE => {
                    Instruction::Add(Add::new(source.unwrap(), operand_width, destination))
                }
                ADDC_OPCODE => {
                    Instruction::Addc(Addc::new(source.unwrap(), operand_width, destination))
                }
                SUBC_OPCODE => {
                    Instruction::Subc(Subc::new(source.unwrap(), operand_width, destination))
                }
                SUB_OPCODE => {
                    Instruction::Sub(Sub::new(source.unwrap(), operand_width, destination))
                }
                CMP_OPCODE => {
                    Instruction::Cmp(Cmp::new(source.unwrap(), operand_width, destination))
                }
                DADD_OPCODE => {
                    Instruction::Dadd(Dadd::new(source.unwrap(), operand_width, destination))
                }
                BIT_OPCODE => {
                    Instruction::Bit(Bit::new(source.unwrap(), operand_width, destination))
                }
                BIC_OPCODE => {
                    Instruction::Bic(Bic::new(source.unwrap(), operand_width, destination))
                }
                BIS_OPCODE => {
                    Instruction::Bis(Bis::new(source.unwrap(), operand_width, destination))
                }
                XOR_OPCODE => {
                    Instruction::Xor(Xor::new(source.unwrap(), operand_width, destination))
                }
                AND_OPCODE => {
                    Instruction::And(And::new(source.unwrap(), operand_width, destination))
                }
                _ => unreachable!(),
            };

            Some(inst)
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
        assert_eq!(decode(&data, 0), None);
    }

    #[test]
    fn jnz() {
        let data = [0x00, 0x20];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jnz(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn negative_jnz() {
        let data = [0xf9, 0x23];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jnz(inst)) => {
                assert_eq!(inst.offset(), -6);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jz() {
        let data = [0x00, 0x24];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jz(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jlo() {
        let data = [0x00, 0x28];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jlo(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jlc() {
        let data = [0x00, 0x2c];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jc(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jn() {
        let data = [0x00, 0x30];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jn(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jge() {
        let data = [0x00, 0x34];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jge(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jl() {
        let data = [0x00, 0x38];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jl(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jmp() {
        let data = [0x00, 0x3c];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Jmp(inst)) => {
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_direct() {
        let data = [0x09, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_direct() {
        let data = [0x49, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_indexed_positive() {
        let data = [0x19, 0x10, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_indexed_negative() {
        let data = [0x19, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_indexed_positive() {
        let data = [0x59, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_indexed_negative() {
        let data = [0x59, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_indirect() {
        let data = [0x29, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_indirect() {
        let data = [0x69, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_indirect_autoincrement() {
        let data = [0x39, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_indirect_autoincrement() {
        let data = [0x79, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rrc(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_direct() {
        let data = [0x89, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Swpb(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indexed_positive() {
        let data = [0x99, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Swpb(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indexed_negative() {
        let data = [0x99, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Swpb(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indirect() {
        let data = [0xa9, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Swpb(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indirect_autoincrement() {
        let data = [0xb9, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Swpb(inst)) => {
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_direct() {
        let data = [0x09, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_direct() {
        let data = [0x49, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_indexed_positive() {
        let data = [0x19, 0x11, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_indexed_negative() {
        let data = [0x19, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_indexed_positive() {
        let data = [0x59, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_indexed_negative() {
        let data = [0x59, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_indirect() {
        let data = [0x29, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_indirect() {
        let data = [0x69, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_indirect_autoincrement() {
        let data = [0x39, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_indirect_autoincrement() {
        let data = [0x79, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Rra(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_direct() {
        let data = [0x89, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Sxt(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indexed_positive() {
        let data = [0x99, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Sxt(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indexed_negative() {
        let data = [0x99, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Sxt(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indirect() {
        let data = [0xa9, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Sxt(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indirect_autoincrement() {
        let data = [0xb9, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Sxt(inst)) => {
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_direct() {
        let data = [0x09, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_direct() {
        let data = [0x49, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_indexed_positive() {
        let data = [0x19, 0x12, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_indexed_negative() {
        let data = [0x19, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_indexed_positive() {
        let data = [0x59, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_indexed_negative() {
        let data = [0x59, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_indirect() {
        let data = [0x29, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_indirect() {
        let data = [0x69, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_indirect_autoincrement() {
        let data = [0x39, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_indirect_autoincrement() {
        let data = [0x79, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 1);
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_direct() {
        let data = [0x89, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterDirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indexed_positive() {
        let data = [0x99, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, 4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indexed_negative() {
        let data = [0x99, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((9, -4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indirect() {
        let data = [0xa9, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::RegisterIndirect(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indirect_autoincrement() {
        let data = [0xb9, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::IndirectAutoIncrement(9));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn reti() {
        let data = [0x00, 0x13];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Reti(inst)) => {
                assert_eq!(inst, Reti::new())
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_one() {
        let data = [0x12, 0x12, 0x0, 0x44];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Absolute(0x4400));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_two() {
        let data = [0x22, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(4));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_three() {
        let data = [0x32, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(8));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_zero() {
        let data = [0x03, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(0));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_one() {
        let data = [0x13, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(1));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_two() {
        let data = [0x23, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(2));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_three() {
        let data = [0x33, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Push(inst)) => {
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.source(), &Source::Constant(-1));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_pc_symbolic() {
        let data = [0x90, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::Symbolic(2));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_pc_immediate() {
        let data = [0xb0, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Call(inst)) => {
                assert_eq!(inst.source(), &Source::Immediate(2));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    // TODO: add real tests here for two operand instructions
    #[test]
    fn mov_test() {
        let data = [0x0f, 0x4f];
        let inst = decode(&data, 0);
    }

    #[test]
    fn mov_index_indexed() {
        let data = [0x9f, 0x4f, 0x2, 0x0, 0x2, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::Mov(inst)) => {
                assert_eq!(inst.source(), &Source::Indexed((15, 2)));
                assert_eq!(inst.operand_width(), 0);
                assert_eq!(inst.destination(), &Destination::Indexed((15, 2)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }
}
