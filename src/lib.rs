use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
struct Instruction {}

/// JMP_MASK masks off the high three bits to check whether the pattern 001
/// is present. This describes a JMP instruction
const JMP_MASK: u16 = 0b0010_0000_0000_0000;

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

        JmpInstruction{
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

fn next_instruction<R: Read>(reader: &mut R) -> Option<JmpInstruction> {
    let mut first_bytes :[u8; 2] = [0; 2];

    match reader.read_exact(&mut first_bytes) {
        Ok(_) => {
            let first_word = u16::from_le_bytes(first_bytes);

            if first_word & JMP_MASK == JMP_MASK {
                // jmp instruction
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
                return Some(inst);
            }

            None
        },
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::next_instruction;
    use crate::JmpCondition;

    #[test]
    fn empty_data() {
        let data= vec![];
        assert_eq!(next_instruction(&mut &data[..]), None);
    }

    #[test]
    fn jnz() {
        let data = vec![0x00, 0x20];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jnz);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn negative_jnz() {
        let data = vec![0xf9, 0x23];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jnz);
        assert_eq!(i.offset(), -6);
    }

    #[test]
    fn jz() {
        let data = vec![0x00, 0x24];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jz);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jlo() {
        let data = vec![0x00, 0x28];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jlo);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jlc() {
        let data = vec![0x00, 0x2c];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jc);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jn() {
        let data = vec![0x00, 0x30];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jn);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jge() {
        let data = vec![0x00, 0x34];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jge);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jl() {
        let data = vec![0x00, 0x38];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jl);
        assert_eq!(i.offset(), 0);
    }

    #[test]
    fn jmp() {
        let data = vec![0x00, 0x3c];
        let inst = next_instruction(&mut &data[..]);
        assert_ne!(inst, None);
        let i = inst.unwrap();
        assert_eq!(i.condition(), JmpCondition::Jmp);
        assert_eq!(i.offset(), 0);
    }
}

