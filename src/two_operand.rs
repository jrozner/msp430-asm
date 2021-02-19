use std::fmt;

use crate::instruction::{BYTE_SUFFIX, WORD_SUFFIX};
use crate::operand::{Destination, HasWidth, OperandWidth, Source};

pub trait TwoOperand {
    fn mnemonic(&self) -> &str;
    fn source(&self) -> &Source;
    fn destination(&self) -> &Destination;
    fn len(&self) -> usize;
}

macro_rules! two_operand {
    ($t:ident, $n:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $t {
            source: Source,
            operand_width: OperandWidth,
            destination: Destination,
        }

        impl $t {
            pub fn new(
                source: Source,
                operand_width: OperandWidth,
                destination: Destination,
            ) -> $t {
                $t {
                    source: source,
                    operand_width: operand_width,
                    destination: destination,
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

            fn source(&self) -> &Source {
                &self.source
            }

            fn destination(&self) -> &Destination {
                &self.destination
            }

            fn len(&self) -> usize {
                2 + self.source.len() + self.destination.len()
            }
        }

        impl HasWidth for $t {
            fn operand_width(&self) -> &OperandWidth {
                &self.operand_width
            }
        }
    };
}

two_operand!(Mov, "mov");

impl fmt::Display for Mov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(0) && self.destination == Destination::RegisterDirect(3)
        {
            return write!(f, "nop");
        }

        if self.source == Source::RegisterIndirectAutoIncrement(1) {
            if self.destination == Destination::RegisterDirect(0) {
                return write!(f, "ret");
            } else {
                return write!(f, "pop {}", self.destination);
            }
        }

        if self.destination == Destination::RegisterDirect(0) {
            return write!(f, "br {}", self.source);
        }

        write!(f, "mov{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Add, "add");

impl fmt::Display for Add {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(1) {
            write!(f, "inc {}", self.destination)
        } else if self.source == Source::Constant(2) {
            write!(f, "incd {}", self.destination)
        } else if self.source == self.destination {
            write!(f, "rla{} {}", suffix, self.destination)
        } else {
            write!(f, "add{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Addc, "addc");

impl fmt::Display for Addc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(0) {
            write!(f, "adc{} {}", suffix, self.destination)
        } else if self.source == self.destination {
            write!(f, "rlc{} {}", suffix, self.destination)
        } else {
            write!(f, "addc{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Subc, "subc");

impl fmt::Display for Subc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(0) {
            write!(f, "sbc{} {}", suffix, self.destination)
        } else {
            write!(f, "subc{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Sub, "sub");

impl fmt::Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(1) {
            write!(f, "dec {}", self.destination)
        } else if self.source == Source::Constant(2) {
            write!(f, "decd {}", self.destination)
        } else {
            write!(f, "sub{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Cmp, "cmp");

impl fmt::Display for Cmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(0) {
            write!(f, "tst{} {}", suffix, self.destination)
        } else {
            write!(f, "cmp{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Dadd, "dadd");

impl fmt::Display for Dadd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(0) {
            write!(f, "dadc{} {}", suffix, self.destination)
        } else {
            write!(f, "dadd{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(Bit, "bit");

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        write!(f, "bit{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Bic, "bic");

impl fmt::Display for Bic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.destination == Destination::RegisterDirect(2) {
            match self.source {
                Source::Constant(1) => return write!(f, "clrc"),
                Source::Constant(2) => return write!(f, "clrn"),
                Source::Constant(4) => return write!(f, "clrz"),
                Source::Constant(8) => return write!(f, "dint"),
                _ => {}
            }
        }

        write!(f, "bic{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Bis, "bis");

impl fmt::Display for Bis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.destination == Destination::RegisterDirect(2) {
            match self.source {
                Source::Constant(1) => return write!(f, "setc"),
                Source::Constant(2) => return write!(f, "setz"),
                Source::Constant(4) => return write!(f, "setn"),
                Source::Constant(8) => return write!(f, "eint"),
                _ => {}
            }
        }

        write!(f, "bis{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Xor, "xor");

impl fmt::Display for Xor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        if self.source == Source::Constant(-1) {
            write!(f, "inv{} {}", suffix, self.destination)
        } else {
            write!(f, "xor{} {}, {}", suffix, self.source, self.destination)
        }
    }
}

two_operand!(And, "and");

impl fmt::Display for And {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == OperandWidth::Byte {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        write!(f, "and{} {}, {}", suffix, self.source, self.destination)
    }
}
