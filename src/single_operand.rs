use crate::Source;

macro_rules! single_operand {
    ($e:ident) => {
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
        }
    };
}

macro_rules! single_operand_width {
    ($e:ident) => {
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
        }
    };
}

single_operand_width!(Rrc);
single_operand!(Swpb);
single_operand_width!(Rra);
single_operand!(Sxt);
single_operand_width!(Push);
single_operand!(Call);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reti {}

impl Reti {
    pub fn new() -> Reti {
        Reti {}
    }
}
