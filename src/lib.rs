use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    JmpInstruction(JmpInstruction),
    SingleOperand(SingleOperand),
}

#[derive(Debug, Clone, PartialEq)]
enum AddressingMode {
    RegisterDirect(u8),
    // maybe we should just pass the operand in the instruction itself?
    Indexed((u8, i16)),
    RegisterIndirect(u8),
    IndirectAutoIncrement(u8),
    Symbolic(i16),
    Immediate(i16),
    Absolute(u16),
    Constant(i8),
}

#[derive(Debug, Clone, PartialEq)]
struct SingleOperand {
    opcode: u16,
    operand_width: u8,
    addressing_mode: Option<AddressingMode>,
}

const RRC_OPCODE: u16 = 0;
const SWPB_OPCODE: u16 = 1;
const RRA_OPCODE: u16 = 2;
const SXT_OPCODE: u16 = 3;
const PUSH_OPCODE: u16 = 4;
const CALL_OPCODE: u16 = 5;
const RETI_OPCODE: u16 = 6;

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

///JmpCondition describes the condition for a Jxx instruction
#[derive(Debug, Clone, Copy, PartialEq)]
enum JmpCondition {
    Jnz,
    Jz,
    Jlo,
    Jc,
    Jn,
    Jge,
    Jl,
    Jmp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct JmpInstruction {
    /// the condition to determine if the jump should happen
    condition: JmpCondition,

    /// the number of words (16 bits) to jump forward or backward
    offset: i16,
}

impl JmpInstruction {
    fn new(condition: JmpCondition, offset: u16) -> JmpInstruction {
        let fixed_offset: i16;
        // msp430 uses one's compliment so we need to do some bit magic to
        // figure out if we have a negative offset and if so convert it into
        // the actual negative number that it is representing
        if offset & 0b10_0000_0000 > 0 {
            fixed_offset = -1 * (0b0000_0011_1111_1111 & !offset) as i16;
        } else {
            fixed_offset = offset as i16;
        }

        JmpInstruction {
            condition: condition,
            offset: fixed_offset,
        }
    }

    fn condition(&self) -> JmpCondition {
        self.condition
    }

    fn offset(&self) -> i16 {
        self.offset
    }
}

fn decode(data: &[u8], addr: usize) -> Option<Instruction> {
    if data.len() < (addr + 2) {
        return None;
    }

    let (int_bytes, _) = data[addr..addr + 2].split_at(std::mem::size_of::<u16>());
    // TODO: do we need to worry about the unwrap failing here?
    let first_word = u16::from_le_bytes(int_bytes.try_into().unwrap());

    let inst_type = first_word & INST_TYPE_MASK;
    match inst_type {
        SINGLE_OPERAND_INSTRUCTION => {
            let opcode = (SINGLE_OPERAND_OPCODE_MASK & first_word) >> 7;
            let register = (SINGLE_OPERAND_REGISTER_MASK & first_word) as u8;
            let source = (SINGLE_OPERAND_SOURCE_MASK & first_word) >> 4;
            let operand_width = ((SINGLE_OPERAND_WIDTH_MASK & first_word) >> 6) as u8;

            // TODO: make sure addr + 4 exists for instructions that have a second operand

            let addressing_mode = match register {
                0 => match source {
                    0 => None, // NOTE: this is a special case for RETI which doesn't follow?
                    1 => {
                        let (int_bytes, _) =
                            data[addr + 2..addr + 4].split_at(std::mem::size_of::<u16>());
                        let second_word =
                            ones_complement(u16::from_le_bytes(int_bytes.try_into().unwrap()));
                        Some(AddressingMode::Symbolic(second_word))
                    }
                    3 => {
                        let (int_bytes, _) =
                            data[addr + 2..addr + 4].split_at(std::mem::size_of::<u16>());
                        let second_word =
                            ones_complement(u16::from_le_bytes(int_bytes.try_into().unwrap()));
                        Some(AddressingMode::Immediate(second_word))
                    }
                    _ => panic!("invalid addressing mode"),
                },
                2 => match source {
                    1 => {
                        let (int_bytes, _) =
                            data[addr + 2..addr + 4].split_at(std::mem::size_of::<u16>());
                        let second_word = u16::from_le_bytes(int_bytes.try_into().unwrap());
                        Some(AddressingMode::Absolute(second_word))
                    }
                    2 => Some(AddressingMode::Constant(4)),
                    3 => Some(AddressingMode::Constant(8)),
                    _ => unreachable!(),
                },
                3 => match source {
                    0 => Some(AddressingMode::Constant(0)),
                    1 => Some(AddressingMode::Constant(1)),
                    2 => Some(AddressingMode::Constant(2)),
                    3 => Some(AddressingMode::Constant(-1)),
                    _ => unreachable!(),
                },
                _ => match source {
                    0 => Some(AddressingMode::RegisterDirect(register)),
                    1 => {
                        let (int_bytes, _) =
                            data[addr + 2..addr + 4].split_at(std::mem::size_of::<u16>());
                        let second_word =
                            ones_complement(u16::from_le_bytes(int_bytes.try_into().unwrap()));
                        Some(AddressingMode::Indexed((register, second_word)))
                    }
                    2 => Some(AddressingMode::RegisterIndirect(register)),
                    3 => Some(AddressingMode::IndirectAutoIncrement(register)),
                    _ => panic!("invalid addressing mode"),
                },
            };

            match opcode {
                RRC_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: RRC_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                SWPB_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: SWPB_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                RRA_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: RRA_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                SXT_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: SXT_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                PUSH_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: PUSH_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                CALL_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: CALL_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                RETI_OPCODE => Some(Instruction::SingleOperand(SingleOperand {
                    opcode: RETI_OPCODE,
                    addressing_mode: addressing_mode,
                    operand_width: operand_width,
                })),
                _ => None,
            }
        }
        JMP_INSTRUCTION => {
            let condition = (first_word & JMP_CONDITION_MASK) >> 10;
            let offset = first_word & JMP_OFFSET;
            // TODO: we may be able to simplify this by using C style
            // enums and just convert from the condition to the value
            // after checking that the condition is [0, 7)
            let inst = match condition {
                0 => JmpInstruction::new(JmpCondition::Jnz, offset),
                1 => JmpInstruction::new(JmpCondition::Jz, offset),
                2 => JmpInstruction::new(JmpCondition::Jlo, offset),
                3 => JmpInstruction::new(JmpCondition::Jc, offset),
                4 => JmpInstruction::new(JmpCondition::Jn, offset),
                5 => JmpInstruction::new(JmpCondition::Jge, offset),
                6 => JmpInstruction::new(JmpCondition::Jl, offset),
                7 => JmpInstruction::new(JmpCondition::Jmp, offset),
                _ => unreachable!(),
            };
            return Some(Instruction::JmpInstruction(inst));
        }
        _ => {
            // The opcode is the first four bits for this type of
            // instruction so there isn't a simple mask we can check.
            // If it doesn't match a single operand or jmp instuction
            // we'll fall through to here and attempt to match a two
            // operand. If it doesn't match any we'll return None
            None
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
        let data = vec![];
        assert_eq!(decode(&data, 0), None);
    }

    #[test]
    fn jnz() {
        let data = vec![0x00, 0x20];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jnz);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn negative_jnz() {
        let data = vec![0xf9, 0x23];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jnz);
                assert_eq!(inst.offset(), -6);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jz() {
        let data = vec![0x00, 0x24];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jz);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jlo() {
        let data = vec![0x00, 0x28];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jlo);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jlc() {
        let data = vec![0x00, 0x2c];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jc);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jn() {
        let data = vec![0x00, 0x30];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jn);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jge() {
        let data = vec![0x00, 0x34];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jge);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jl() {
        let data = vec![0x00, 0x38];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jl);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn jmp() {
        let data = vec![0x00, 0x3c];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::JmpInstruction(inst)) => {
                assert_eq!(inst.condition(), JmpCondition::Jmp);
                assert_eq!(inst.offset(), 0);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_direct() {
        let data = vec![0x09, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_direct() {
        let data = vec![0x49, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_indexed_positive() {
        let data = vec![0x19, 0x10, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_indexed_negative() {
        let data = vec![0x19, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_indexed_positive() {
        let data = vec![0x59, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_indexed_negative() {
        let data = vec![0x59, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_indirect() {
        let data = vec![0x29, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_indirect() {
        let data = vec![0x69, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_w_register_indirect_autoincrement() {
        let data = vec![0x39, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rrc_b_register_indirect_autoincrement() {
        let data = vec![0x79, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRC_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_direct() {
        let data = vec![0x89, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SWPB_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indexed_positive() {
        let data = vec![0x99, 0x10, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SWPB_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indexed_negative() {
        let data = vec![0x99, 0x10, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SWPB_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indirect() {
        let data = vec![0xa9, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SWPB_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn swpb_register_indirect_autoincrement() {
        let data = vec![0xb9, 0x10];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SWPB_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_direct() {
        let data = vec![0x09, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_direct() {
        let data = vec![0x49, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_indexed_positive() {
        let data = vec![0x19, 0x11, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_indexed_negative() {
        let data = vec![0x19, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_indexed_positive() {
        let data = vec![0x59, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_indexed_negative() {
        let data = vec![0x59, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_indirect() {
        let data = vec![0x29, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_indirect() {
        let data = vec![0x69, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_w_register_indirect_autoincrement() {
        let data = vec![0x39, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn rra_b_register_indirect_autoincrement() {
        let data = vec![0x79, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RRA_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_direct() {
        let data = vec![0x89, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SXT_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indexed_positive() {
        let data = vec![0x99, 0x11, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SXT_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indexed_negative() {
        let data = vec![0x99, 0x11, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SXT_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indirect() {
        let data = vec![0xa9, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SXT_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn sxt_register_indirect_autoincrement() {
        let data = vec![0xb9, 0x11];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, SXT_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_direct() {
        let data = vec![0x09, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_direct() {
        let data = vec![0x49, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_indexed_positive() {
        let data = vec![0x19, 0x12, 0x4, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_indexed_negative() {
        let data = vec![0x19, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_indexed_positive() {
        let data = vec![0x59, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_indexed_negative() {
        let data = vec![0x59, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_indirect() {
        let data = vec![0x29, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_indirect() {
        let data = vec![0x69, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_w_register_indirect_autoincrement() {
        let data = vec![0x39, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_b_register_indirect_autoincrement() {
        let data = vec![0x79, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 1);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_direct() {
        let data = vec![0x89, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterDirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indexed_positive() {
        let data = vec![0x99, 0x12, 0x04, 0x00];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, 4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indexed_negative() {
        let data = vec![0x99, 0x12, 0xfb, 0xff];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Indexed((9, -4))));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indirect() {
        let data = vec![0xa9, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::RegisterIndirect(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_register_indirect_autoincrement() {
        let data = vec![0xb9, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(
                    inst.addressing_mode,
                    Some(AddressingMode::IndirectAutoIncrement(9))
                );
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn reti() {
        let data = vec![0x00, 0x13];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, RETI_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, None);
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_one() {
        let data = vec![0x12, 0x12, 0x0, 0x44];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Absolute(0x4400)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_two() {
        let data = vec![0x22, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(4)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_sr_three() {
        let data = vec![0x32, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(8)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_zero() {
        let data = vec![0x03, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(0)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_one() {
        let data = vec![0x13, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(1)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_two() {
        let data = vec![0x23, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(2)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn push_const_cg_three() {
        let data = vec![0x33, 0x12];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, PUSH_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Constant(-1)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_pc_symbolic() {
        let data = vec![0x90, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Symbolic(2)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }

    #[test]
    fn call_pc_immediate() {
        let data = vec![0xb0, 0x12, 0x2, 0x0];
        let inst = decode(&data, 0);
        match inst {
            None => panic!("no instruction returned"),
            Some(Instruction::SingleOperand(inst)) => {
                assert_eq!(inst.opcode, CALL_OPCODE);
                assert_eq!(inst.operand_width, 0);
                assert_eq!(inst.addressing_mode, Some(AddressingMode::Immediate(2)));
            }
            Some(inst) => panic!(format!("invalid instruction decoded: {:?}", inst)),
        }
    }
}
