use std::fmt;

use crate::instruction::{BYTE_SUFFIX, WORD_SUFFIX};
use crate::operand::{Destination, Source};

macro_rules! two_operand {
    ($e:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $e {
            source: Source,
            operand_width: u8,
            destination: Destination,
        }

        impl $e {
            pub fn new(source: Source, operand_width: u8, destination: Destination) -> $e {
                $e {
                    source: source,
                    operand_width: operand_width,
                    destination: destination,
                }
            }

            pub fn source(&self) -> &Source {
                &self.source
            }

            pub fn operand_width(&self) -> u8 {
                self.operand_width
            }

            pub fn destination(&self) -> &Destination {
                &self.destination
            }

            pub fn len(&self) -> usize {
                2 + self.source.len() + self.destination.len()
            }
        }
    };
}

two_operand!(Mov);

impl fmt::Display for Mov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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
            return write!(f, "br {}", self.destination);
        }

        write!(f, "mov{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Add);

impl fmt::Display for Add {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Addc);

impl fmt::Display for Addc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Subc);

impl fmt::Display for Subc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Sub);

impl fmt::Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Cmp);

impl fmt::Display for Cmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Dadd);

impl fmt::Display for Dadd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Bit);

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        write!(f, "bit{} {}, {}", suffix, self.source, self.destination)
    }
}

two_operand!(Bic);

impl fmt::Display for Bic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Bis);

impl fmt::Display for Bis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(Xor);

impl fmt::Display for Xor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
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

two_operand!(And);

impl fmt::Display for And {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.operand_width == 1 {
            BYTE_SUFFIX
        } else {
            WORD_SUFFIX
        };

        write!(f, "and{} {}, {}", suffix, self.source, self.destination)
    }
}
