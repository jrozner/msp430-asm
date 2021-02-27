use crate::instruction::Instruction;
use crate::operand::{Operand, OperandWidth};

use crate::two_operand::*;
use std::fmt;

pub trait Emulate {
    fn emulate(self) -> Option<Instruction>;
}

pub trait Emulated {
    fn mnemonic(&self) -> &str;
    fn destination(&self) -> &Option<Operand>;
    fn size(&self) -> usize;
    fn operand_width(&self) -> &Option<OperandWidth>;
}

macro_rules! emulated {
    ($t:ident, $n:expr, $o:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $t {
            destination: Option<Operand>,
            operand_width: Option<OperandWidth>,
            // we need to store the original instruction because emulation
            // does not keep the original source and destination which makes
            // it a lossy process. There are certain instructions where the
            // source could use different addressing modes or that can be
            // assembled in multiple ways
            // (eg. mov #0, r15; [using immediate 0x0000 or constant #0])
            original: $o,
        }

        impl $t {
            pub fn new(
                destination: Option<Operand>,
                operand_width: Option<OperandWidth>,
                original: $o,
            ) -> $t {
                $t {
                    destination,
                    operand_width,
                    original,
                }
            }
        }

        impl Emulated for $t {
            fn mnemonic(&self) -> &str {
                match self.operand_width {
                    Some(OperandWidth::Word) | None => $n,
                    Some(OperandWidth::Byte) => concat!($n, ".b"),
                }
            }

            fn destination(&self) -> &Option<Operand> {
                &self.destination
            }

            fn size(&self) -> usize {
                self.original.size()
            }

            fn operand_width(&self) -> &Option<OperandWidth> {
                &self.operand_width
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if self.destination.is_none() && self.operand_width.is_none() {
                    write!(f, "{}", $n)
                } else {
                    write!(f, "{} {}", self.mnemonic(), self.destination.unwrap())
                }
            }
        }
    };
}

emulated!(Adc, "adc", Addc);
emulated!(Br, "br", Mov);
emulated!(Clr, "clr", Mov);
emulated!(Clrc, "clrc", Bic);
emulated!(Clrn, "clrn", Bic);
emulated!(Clrz, "clrz", Bic);
emulated!(Dadc, "dadc", Dadd);
emulated!(Dec, "dec", Sub);
emulated!(Decd, "decd", Sub);
emulated!(Dint, "dint", Bic);
emulated!(Eint, "eint", Bis);
emulated!(Inc, "inc", Add);
emulated!(Incd, "incd", Add);
emulated!(Inv, "inv", Xor);
emulated!(Nop, "nop", Mov);
emulated!(Pop, "pop", Mov);
emulated!(Ret, "ret", Mov);
emulated!(Rla, "rla", Add);
emulated!(Rlc, "rlc", Addc);
emulated!(Sbc, "sbc", Subc);
emulated!(Setc, "setc", Bis);
emulated!(Setn, "Setn", Bis);
emulated!(Setz, "setz", Bis);
emulated!(Tst, "tst", Cmp);
