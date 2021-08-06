use std::fmt;

use crate::emulate;
use crate::emulate::Emulate;
use crate::instruction::Instruction;
use crate::operand::{Operand, OperandWidth};

/// All two operand instructions implement this trait to provide a common
/// interface and polymorphism
pub trait TwoOperand {
    /// Return the mnemonic for the instruction. This is operand width aware
    fn mnemonic(&self) -> &str;
    /// Returns the source operand
    fn source(&self) -> &Operand;
    /// Returns the destination operand
    fn destination(&self) -> &Operand;
    /// Returns the size of the instruction (in bytes)
    fn size(&self) -> usize;
    /// Returns the operand width
    fn operand_width(&self) -> &OperandWidth;
}

macro_rules! two_operand {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $t {
            source: Operand,
            operand_width: OperandWidth,
            destination: Operand,
        }

        impl $t {
            pub fn new(source: Operand, operand_width: OperandWidth, destination: Operand) -> $t {
                $t {
                    source,
                    operand_width,
                    destination,
                }
            }
        }

        impl TwoOperand for $t {
            fn mnemonic(&self) -> &str {
                match self.operand_width {
                    OperandWidth::Word => $n,
                    OperandWidth::Byte => concat!($n, ".b"),
                }
            }

            fn source(&self) -> &Operand {
                &self.source
            }

            fn destination(&self) -> &Operand {
                &self.destination
            }

            fn size(&self) -> usize {
                2 + self.source.size() + self.destination.size()
            }

            fn operand_width(&self) -> &OperandWidth {
                &self.operand_width
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{} {}, {}",
                    self.mnemonic(),
                    self.source,
                    self.destination
                )
            }
        }
    };
}

two_operand!(Mov, "mov");

impl Emulate for Mov {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(0) && self.destination == Operand::RegisterDirect(3) {
            return Some(Instruction::Nop(emulate::Nop::new(None, None, *self)));
        }

        if self.source == Operand::Constant(0) || self.source == Operand::Immediate(0) {
            if let Operand::RegisterDirect(_) = self.destination {
                return Some(Instruction::Clr(emulate::Clr::new(
                    Some(self.destination),
                    None,
                    *self,
                )));
            }
        }

        if self.source == Operand::RegisterIndirectAutoIncrement(1) {
            if self.destination == Operand::RegisterDirect(0) {
                return Some(Instruction::Ret(emulate::Ret::new(None, None, *self)));
            } else {
                return Some(Instruction::Pop(emulate::Pop::new(
                    Some(self.destination),
                    Some(self.operand_width),
                    *self,
                )));
            }
        }

        if self.destination == Operand::RegisterDirect(0) {
            return Some(Instruction::Br(emulate::Br::new(
                Some(self.source),
                None,
                *self,
            )));
        }

        None
    }
}

two_operand!(Add, "add");

impl Emulate for Add {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(1) {
            Some(Instruction::Inc(emulate::Inc::new(
                Some(self.destination),
                None,
                *self,
            )))
        } else if self.source == Operand::Constant(2) {
            Some(Instruction::Incd(emulate::Incd::new(
                Some(self.destination),
                None,
                *self,
            )))
        } else if self.source == self.destination {
            Some(Instruction::Rla(emulate::Rla::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Addc, "addc");

impl Emulate for Addc {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(0) {
            Some(Instruction::Adc(emulate::Adc::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else if self.source == self.destination {
            Some(Instruction::Rlc(emulate::Rlc::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Subc, "subc");

impl Emulate for Subc {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(0) {
            Some(Instruction::Sbc(emulate::Sbc::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Sub, "sub");

impl Emulate for Sub {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(1) {
            Some(Instruction::Dec(emulate::Dec::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else if self.source == Operand::Constant(2) {
            Some(Instruction::Decd(emulate::Decd::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Cmp, "cmp");

impl Emulate for Cmp {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(0) {
            Some(Instruction::Tst(emulate::Tst::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Dadd, "dadd");

impl Emulate for Dadd {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(0) {
            Some(Instruction::Dadc(emulate::Dadc::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(Bit, "bit");
two_operand!(Bic, "bic");

impl Emulate for Bic {
    fn emulate(&self) -> Option<Instruction> {
        if self.destination == Operand::RegisterDirect(2) {
            match self.source {
                Operand::Constant(1) => {
                    return Some(Instruction::Clrc(emulate::Clrc::new(None, None, *self)))
                }
                Operand::Constant(2) => {
                    return Some(Instruction::Clrn(emulate::Clrn::new(None, None, *self)))
                }
                Operand::Constant(4) => {
                    return Some(Instruction::Clrz(emulate::Clrz::new(None, None, *self)))
                }
                Operand::Constant(8) => {
                    return Some(Instruction::Dint(emulate::Dint::new(None, None, *self)))
                }
                _ => {}
            }
        }

        None
    }
}

two_operand!(Bis, "bis");

impl Emulate for Bis {
    fn emulate(&self) -> Option<Instruction> {
        if self.destination == Operand::RegisterDirect(2) {
            match self.source {
                Operand::Constant(1) => {
                    return Some(Instruction::Setc(emulate::Setc::new(None, None, *self)))
                }
                Operand::Constant(2) => {
                    return Some(Instruction::Setz(emulate::Setz::new(None, None, *self)))
                }
                Operand::Constant(4) => {
                    return Some(Instruction::Setn(emulate::Setn::new(None, None, *self)))
                }
                Operand::Constant(8) => {
                    return Some(Instruction::Eint(emulate::Eint::new(None, None, *self)))
                }
                _ => {}
            }
        }

        None
    }
}

two_operand!(Xor, "xor");

impl Emulate for Xor {
    fn emulate(&self) -> Option<Instruction> {
        if self.source == Operand::Constant(-1) {
            Some(Instruction::Inv(emulate::Inv::new(
                Some(self.destination),
                Some(self.operand_width),
                *self,
            )))
        } else {
            None
        }
    }
}

two_operand!(And, "and");
