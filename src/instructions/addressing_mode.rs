#[derive(Debug, Clone, PartialEq)]
pub enum AddressingMode {
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
