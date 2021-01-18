use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    JmpInstruction(JmpInstruction),
    SingleOperand(SingleOperand),
}

#[derive(Debug, Clone, PartialEq)]
struct SingleOperand {}

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
        SINGLE_OPERAND_INSTRUCTION => None,
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
}
