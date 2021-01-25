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
        }
    };
}

two_operand!(Mov);
two_operand!(Add);
two_operand!(Addc);
two_operand!(Subc);
two_operand!(Sub);
two_operand!(Cmp);
two_operand!(Dadd);
two_operand!(Bit);
two_operand!(Bic);
two_operand!(Bis);
two_operand!(Xor);
two_operand!(And);
