pub mod addressing_mode;
pub mod jxx;
pub mod single_operand;

pub use addressing_mode::AddressingMode;
pub use jxx::*;
pub use single_operand::*;

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
}
