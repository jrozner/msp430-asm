use std::fmt;

use crate::jxx::*;
use crate::single_operand::*;
use crate::two_operand::*;

pub const BYTE_SUFFIX: &str = ".b";
pub const WORD_SUFFIX: &str = "";

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // single operand instructions
    Rrc(Rrc),
    Swpb(Swpb),
    Rra(Rra),
    Sxt(Sxt),
    Push(Push),
    Call(Call),
    Reti(Reti),

    // Jxx instructions
    Jnz(Jnz),
    Jz(Jz),
    Jlo(Jlo),
    Jc(Jc),
    Jn(Jn),
    Jge(Jge),
    Jl(Jl),
    Jmp(Jmp),

    // two operand instructions
    Mov(Mov),
    Add(Add),
    Addc(Addc),
    Subc(Subc),
    Sub(Sub),
    Cmp(Cmp),
    Dadd(Dadd),
    Bit(Bit),
    Bic(Bic),
    Bis(Bis),
    Xor(Xor),
    And(And),
}

impl Instruction {
    pub fn len(&self) -> usize {
        match self {
            Instruction::Rrc(inst) => inst.len(),
            Instruction::Swpb(inst) => inst.len(),
            Instruction::Rra(inst) => inst.len(),
            Instruction::Sxt(inst) => inst.len(),
            Instruction::Push(inst) => inst.len(),
            Instruction::Call(inst) => inst.len(),
            Instruction::Reti(inst) => inst.len(),
            Instruction::Jnz(inst) => inst.len(),
            Instruction::Jz(inst) => inst.len(),
            Instruction::Jlo(inst) => inst.len(),
            Instruction::Jc(inst) => inst.len(),
            Instruction::Jn(inst) => inst.len(),
            Instruction::Jge(inst) => inst.len(),
            Instruction::Jl(inst) => inst.len(),
            Instruction::Jmp(inst) => inst.len(),
            Instruction::Mov(inst) => inst.len(),
            Instruction::Add(inst) => inst.len(),
            Instruction::Addc(inst) => inst.len(),
            Instruction::Subc(inst) => inst.len(),
            Instruction::Sub(inst) => inst.len(),
            Instruction::Cmp(inst) => inst.len(),
            Instruction::Dadd(inst) => inst.len(),
            Instruction::Bit(inst) => inst.len(),
            Instruction::Bic(inst) => inst.len(),
            Instruction::Bis(inst) => inst.len(),
            Instruction::Xor(inst) => inst.len(),
            Instruction::And(inst) => inst.len(),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Rrc(inst) => write!(f, "{}", inst),
            Instruction::Swpb(inst) => write!(f, "{}", inst),
            Instruction::Rra(inst) => write!(f, "{}", inst),
            Instruction::Sxt(inst) => write!(f, "{}", inst),
            Instruction::Push(inst) => write!(f, "{}", inst),
            Instruction::Call(inst) => write!(f, "{}", inst),
            Instruction::Reti(inst) => write!(f, "{}", inst),
            Instruction::Jnz(inst) => write!(f, "{}", inst),
            Instruction::Jz(inst) => write!(f, "{}", inst),
            Instruction::Jlo(inst) => write!(f, "{}", inst),
            Instruction::Jc(inst) => write!(f, "{}", inst),
            Instruction::Jn(inst) => write!(f, "{}", inst),
            Instruction::Jge(inst) => write!(f, "{}", inst),
            Instruction::Jl(inst) => write!(f, "{}", inst),
            Instruction::Jmp(inst) => write!(f, "{}", inst),
            Instruction::Mov(inst) => write!(f, "{}", inst),
            Instruction::Add(inst) => write!(f, "{}", inst),
            Instruction::Addc(inst) => write!(f, "{}", inst),
            Instruction::Subc(inst) => write!(f, "{}", inst),
            Instruction::Sub(inst) => write!(f, "{}", inst),
            Instruction::Cmp(inst) => write!(f, "{}", inst),
            Instruction::Dadd(inst) => write!(f, "{}", inst),
            Instruction::Bit(inst) => write!(f, "{}", inst),
            Instruction::Bic(inst) => write!(f, "{}", inst),
            Instruction::Bis(inst) => write!(f, "{}", inst),
            Instruction::Xor(inst) => write!(f, "{}", inst),
            Instruction::And(inst) => write!(f, "{}", inst),
        }
    }
}
