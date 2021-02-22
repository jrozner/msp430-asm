use crate::operand::{OperandWidth, Source};

use std::fmt;

pub trait SingleOperand {
    fn mnemonic(&self) -> &str;
    fn source(&self) -> &Source;
    fn len(&self) -> usize;
    fn operand_width(&self) -> &Option<OperandWidth>;
}

macro_rules! single_operand {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $t {
            source: Source,
            operand_width: Option<OperandWidth>,
        }

        impl $t {
            pub fn new(source: Source, operand_width: Option<OperandWidth>) -> $t {
                $t {
                    source: source,
                    operand_width: operand_width,
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

            fn source(&self) -> &Source {
                &self.source
            }

            fn len(&self) -> usize {
                2 + self.source.len()
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reti {}

impl Reti {
    pub fn new() -> Reti {
        Reti {}
    }

    pub fn len(&self) -> usize {
        2
    }
}

impl fmt::Display for Reti {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reti")
    }
}
