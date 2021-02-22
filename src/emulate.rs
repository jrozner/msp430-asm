use crate::instruction::Instruction;
use crate::operand::{Operand, OperandWidth};

use std::fmt;

pub trait Emulate {
    fn emulate(&self) -> Option<Instruction>;
}

pub trait Emulated {
    fn mnemonic(&self) -> &str;
    fn destination(&self) -> &Option<Operand>;
    fn len(&self) -> usize;
    fn operand_width(&self) -> &Option<OperandWidth>;
}

macro_rules! emulated {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $t {
            destination: Option<Operand>,
            operand_width: Option<OperandWidth>,
            // we need to store the size because emulation does not keep the
            // original source and destination which makes it a lossy
            // process. There are certain instructions where the source could
            // use different addressing modes or that can be assembled in
            // multiple ways
            // (eg. mov #0, r15; [using immediate 0x0000 or constant #0])
            len: usize,
        }

        impl $t {
            pub fn new(
                destination: Option<Operand>,
                operand_width: Option<OperandWidth>,
                len: usize,
            ) -> $t {
                $t {
                    destination: destination,
                    operand_width: operand_width,
                    len: len,
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

            fn len(&self) -> usize {
                self.len
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

emulated!(Adc, "adc");
emulated!(Br, "br");
emulated!(Clr, "clr");
emulated!(Clrc, "clrc");
emulated!(Clrn, "clrn");
emulated!(Clrz, "clrz");
emulated!(Dadc, "dadc");
emulated!(Dec, "dec");
emulated!(Decd, "decd");
emulated!(Dint, "dint");
emulated!(Eint, "eint");
emulated!(Inc, "inc");
emulated!(Incd, "incd");
emulated!(Inv, "inv");
emulated!(Nop, "nop");
emulated!(Pop, "pop");
emulated!(Ret, "ret");
emulated!(Rla, "rla");
emulated!(Rlc, "rlc");
emulated!(Sbc, "sbc");
emulated!(Setc, "setc");
emulated!(Setn, "Setn");
emulated!(Setz, "setz");
emulated!(Tst, "tst");
