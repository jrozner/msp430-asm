use crate::AddressingMode;

#[derive(Debug, Clone, PartialEq)]
pub struct Rrc {
    addressing_mode: AddressingMode,
    operand_width: u8,
}

impl Rrc {
    pub fn new(addressing_mode: AddressingMode, operand_width: u8) -> Rrc {
        Rrc {
            addressing_mode: addressing_mode,
            operand_width: operand_width,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }

    pub fn operand_width(&self) -> u8 {
        self.operand_width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Swpb {
    addressing_mode: AddressingMode,
}

impl Swpb {
    pub fn new(addressing_mode: AddressingMode) -> Swpb {
        Swpb {
            addressing_mode: addressing_mode,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rra {
    addressing_mode: AddressingMode,
    operand_width: u8,
}

impl Rra {
    pub fn new(addressing_mode: AddressingMode, operand_width: u8) -> Rra {
        Rra {
            addressing_mode: addressing_mode,
            operand_width: operand_width,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }

    pub fn operand_width(&self) -> u8 {
        self.operand_width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sxt {
    addressing_mode: AddressingMode,
}

impl Sxt {
    pub fn new(addressing_mode: AddressingMode) -> Sxt {
        Sxt {
            addressing_mode: addressing_mode,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Push {
    addressing_mode: AddressingMode,
    operand_width: u8,
}

impl Push {
    pub fn new(addressing_mode: AddressingMode, operand_width: u8) -> Push {
        Push {
            addressing_mode: addressing_mode,
            operand_width: operand_width,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }

    pub fn operand_width(&self) -> u8 {
        self.operand_width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    addressing_mode: AddressingMode,
}

impl Call {
    pub fn new(addressing_mode: AddressingMode) -> Call {
        Call {
            addressing_mode: addressing_mode,
        }
    }

    pub fn addressing_mode(&self) -> &AddressingMode {
        &self.addressing_mode
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reti {}

impl Reti {
    pub fn new() -> Reti {
        Reti {}
    }
}
