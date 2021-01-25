pub fn jxx_fix_offset(offset: u16) -> i16 {
    if offset & 0b10_0000_0000 > 0 {
        -1 * (0b0000_0011_1111_1111 & !offset) as i16
    } else {
        offset as i16
    }
}

macro_rules! jxx {
    ($e:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $e {
            offset: i16,
        }

        impl $e {
            pub fn new(offset: i16) -> $e {
                $e { offset: offset }
            }

            pub fn offset(&self) -> i16 {
                self.offset
            }
        }
    };
}

jxx!(Jnz);
jxx!(Jz);
jxx!(Jlo);
jxx!(Jc);
jxx!(Jn);
jxx!(Jge);
jxx!(Jl);
jxx!(Jmp);
