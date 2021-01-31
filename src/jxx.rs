use std::fmt;

pub fn jxx_fix_offset(offset: u16) -> i16 {
    if offset & 0b10_0000_0000 > 0 {
        -1 * (0b0000_0011_1111_1111 & !offset) as i16
    } else {
        offset as i16
    }
}

macro_rules! jxx {
    ($e:ident, $n:expr) => {
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

        impl fmt::Display for $e {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // LowerHex will treat hex numbers as unsigned so rather than
                // -0x6 we get 0xfffa. This is expected functionality and
                // unlikely to change. This is a working hack for now but we
                // should probably implement a better fix that is more
                // efficient https://github.com/rust-lang/rust/issues/42860
                if self.offset < 0 {
                    write!(f, "{} #-{:#x}", $n, self.offset * -1)
                } else {
                    write!(f, "{} #{:#x}", $n, self.offset)
                }
            }
        }
    };
}

jxx!(Jnz, "jnz");
jxx!(Jz, "jz");
jxx!(Jlo, "jlo");
jxx!(Jc, "jc");
jxx!(Jn, "jn");
jxx!(Jge, "jge");
jxx!(Jl, "jl");
jxx!(Jmp, "jmp");
