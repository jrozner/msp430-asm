use crate::jxx::*;
use crate::single_operand::*;
use crate::two_operand::*;

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
