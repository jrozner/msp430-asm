use std::fmt;

use crate::instruction::{BYTE_SUFFIX, WORD_SUFFIX};
use crate::operand::{HasWidth, OperandWidth, Source};

pub trait SingleOperand {
    fn mnemonic(&self) -> &str;
    fn source(&self) -> &Source;
    fn len(&self) -> usize;
}

macro_rules! single_operand {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $t {
            source: Source,
        }

        impl $t {
            pub fn new(source: Source) -> $t {
                $t { source: source }
            }
        }

        impl SingleOperand for $t {
            fn mnemonic(&self) -> &str {
                $n
            }

            fn source(&self) -> &Source {
                &self.source
            }

            fn len(&self) -> usize {
                2 + self.source.len()
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", $n, self.source)
            }
        }
    };
}

macro_rules! single_operand_width {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $t {
            source: Source,
            operand_width: OperandWidth,
        }

        impl $t {
            pub fn new(source: Source, operand_width: OperandWidth) -> $t {
                $t {
                    source: source,
                    operand_width: operand_width,
                }
            }
        }

        impl SingleOperand for $t {
            fn mnemonic(&self) -> &str {
                match self.operand_width {
                    OperandWidth::Word => $n,
                    OperandWidth::Byte => concat!($n, ".b"),
                }
            }

            fn source(&self) -> &Source {
                &self.source
            }

            fn len(&self) -> usize {
                2 + self.source.len()
            }
        }

        impl HasWidth for $t {
            fn operand_width(&self) -> &OperandWidth {
                &self.operand_width
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let suffix = if self.operand_width == OperandWidth::Byte {
                    BYTE_SUFFIX
                } else {
                    WORD_SUFFIX
                };

                write!(f, "{}{} {}", $n, suffix, self.source)
            }
        }
    };
}

single_operand_width!(Rrc, "rrc");
single_operand!(Swpb, "swpb");
single_operand_width!(Rra, "rra");
single_operand!(Sxt, "sxt");
single_operand_width!(Push, "push");
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
