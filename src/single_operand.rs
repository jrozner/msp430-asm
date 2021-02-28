use crate::operand::{Operand, OperandWidth};

use std::fmt;

/// All single operand instructions implement this trait to provide a common
/// interface and polymorphism
pub trait SingleOperand {
    /// Return the mnemonic for the instruction. This is operand width aware
    fn mnemonic(&self) -> &str;
    /// Returns the source operand
    fn source(&self) -> &Operand;
    /// Returns the size of the instruction (in bytes)
    fn size(&self) -> usize;
    /// Returns the operand width if one is specified
    fn operand_width(&self) -> &Option<OperandWidth>;
}

macro_rules! single_operand {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $t {
            source: Operand,
            operand_width: Option<OperandWidth>,
        }

        impl $t {
            pub fn new(source: Operand, operand_width: Option<OperandWidth>) -> $t {
                $t {
                    source,
                    operand_width,
                }
            }
        }

        impl SingleOperand for $t {
            fn mnemonic(&self) -> &str {
                match self.operand_width {
                    Some(OperandWidth::Word) | None => $n,
                    Some(OperandWidth::Byte) => concat!($n, ".b"),
                }
            }

            fn source(&self) -> &Operand {
                &self.source
            }

            fn size(&self) -> usize {
                2 + self.source.size()
            }

            fn operand_width(&self) -> &Option<OperandWidth> {
                &self.operand_width
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.mnemonic(), self.source)
            }
        }
    };
}

single_operand!(Rrc, "rrc");
single_operand!(Swpb, "swpb");
single_operand!(Rra, "rra");
single_operand!(Sxt, "sxt");
single_operand!(Push, "push");
single_operand!(Call, "call");

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Reti {}

impl Reti {
    pub fn new() -> Reti {
        Reti {}
    }

    pub fn size(&self) -> usize {
        2
    }
}

impl fmt::Display for Reti {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reti")
    }
}
