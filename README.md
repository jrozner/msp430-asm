# msp430-asm

A disassembly engine written in Rust for the msp430.

## Use

There is only one function exposed that you need to care about for disassembling instructions: `decode`.

```rust
extern crate msp430_asm;

use msp430_asm::decode;

let data = [0xf9, 0x23];

match decode(&data) {
    Ok(inst) => println!("{}", inst),
    Err(e) => println!("error decoding instruction: {}", e),
}
```

## License

This project is licensed under the terms of the [MIT](LICENSE) open source license