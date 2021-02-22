use crate::instruction::Instruction;
use crate::operand::{Destination, OperandWidth};

use std::fmt;

pub trait Emulate {
    fn emulate(&self) -> Option<Instruction>;
}

pub trait Emulated {
    fn mnemonic(&self) -> &str;
    fn destination(&self) -> &Option<Destination>;
    fn len(&self) -> usize;
    fn operand_width(&self) -> &Option<OperandWidth>;
}

macro_rules! emulated {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $t {
            destination: Option<Destination>,
            operand_width: Option<OperandWidth>,
        }

        impl $t {
            pub fn new(
                destination: Option<Destination>,
                operand_width: Option<OperandWidth>,
            ) -> $t {
                $t {
                    destination: destination,
                    operand_width: operand_width,
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

            fn destination(&self) -> &Option<Destination> {
                &self.destination
            }

            fn len(&self) -> usize {
                match self.destination {
                    Some(d) => 2 + d.len(),
                    None => 2,
                }
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
