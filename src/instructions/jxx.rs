pub fn jxx_fix_offset(offset: u16) -> i16 {
    if offset & 0b10_0000_0000 > 0 {
        -1 * (0b0000_0011_1111_1111 & !offset) as i16
    } else {
        offset as i16
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jnz {
    offset: i16,
}

impl Jnz {
    pub fn new(offset: i16) -> Jnz {
        Jnz { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jz {
    offset: i16,
}

impl Jz {
    pub fn new(offset: i16) -> Jz {
        Jz { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jlo {
    offset: i16,
}

impl Jlo {
    pub fn new(offset: i16) -> Jlo {
        Jlo { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jc {
    offset: i16,
}

impl Jc {
    pub fn new(offset: i16) -> Jc {
        Jc { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jn {
    offset: i16,
}

impl Jn {
    pub fn new(offset: i16) -> Jn {
        Jn { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jge {
    offset: i16,
}

impl Jge {
    pub fn new(offset: i16) -> Jge {
        Jge { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jl {
    offset: i16,
}

impl Jl {
    pub fn new(offset: i16) -> Jl {
        Jl { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jmp {
    offset: i16,
}

impl Jmp {
    pub fn new(offset: i16) -> Jmp {
        Jmp { offset: offset }
    }

    pub fn offset(&self) -> i16 {
        self.offset
    }
}
