use std::fmt;

use crate::instruction::{BYTE_SUFFIX, WORD_SUFFIX};
use crate::Source;

macro_rules! single_operand {
    ($e:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $e {
            source: Source,
        }

        impl $e {
            pub fn new(source: Source) -> $e {
                $e { source: source }
            }

            pub fn source(&self) -> &Source {
                &self.source
            }

            pub fn len(&self) -> usize {
                2 + self.source.len()
            }
        }

        impl fmt::Display for $e {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", $n, self.source)
            }
        }
    };
}

macro_rules! single_operand_width {
    ($e:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $e {
            source: Source,
            operand_width: u8,
        }

        impl $e {
            pub fn new(source: Source, operand_width: u8) -> $e {
                $e {
                    source: source,
                    operand_width: operand_width,
                }
            }

            pub fn source(&self) -> &Source {
                &self.source
            }

            pub fn operand_width(&self) -> u8 {
                self.operand_width
            }

            pub fn len(&self) -> usize {
                2 + self.source.len()
            }
        }

        impl fmt::Display for $e {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let suffix = if self.operand_width == 1 {
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
}

impl fmt::Display for Reti {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reti")
    }
}
