# customasm
`customasm` is an assembler that allows you to provide your own **custom
instruction sets** to assemble your source files! 
It can be useful, for example, if you're trying to test the bytecode of a new virtual machine,
or if you're eager to write programs for that new microprocessor architecture 
you just implemented in an FPGA chip!

[![crates.io][badge-cratesio-img]][badge-cratesio-url]
[![Latest Release][badge-latest-img]][badge-latest-url]
[![Releases][badge-downloads-img]][badge-downloads-url]

[![Discord][badge-discord-img]][badge-discord-url]

[badge-cratesio-img]: https://img.shields.io/crates/v/customasm
[badge-cratesio-url]: https://crates.io/crates/customasm

[badge-latest-img]: https://img.shields.io/github/v/release/hlorenzi/customasm
[badge-latest-url]: https://github.com/hlorenzi/customasm/releases

[badge-downloads-img]: https://img.shields.io/github/downloads/hlorenzi/customasm/total
[badge-downloads-url]: https://github.com/hlorenzi/customasm/releases

[badge-discord-img]: https://img.shields.io/discord/394999035540275222?label=Join%20the%20Discord%20server!&logo=discord
[badge-discord-url]: https://discord.com/invite/pXeDXGD

[🖥️ Try it right now on your web browser!](https://hlorenzi.github.io/customasm/web/)

[🕹️ Check out an example project](/examples/nes_colors.asm) which targets the NES!

[⌨️ Install the VSCode syntax highlight extension!](https://marketplace.visualstudio.com/items?itemName=hlorenzi.customasm-vscode)

[❤️ Support the author!](https://accounts.hlorenzi.com/supporters)

## Documentation

[📚 Check out the wiki](https://github.com/hlorenzi/customasm/wiki)
for a changelog, documentation, and a how-to-start guide!

[💲 Check out the command-line help!](/src/usage_help.md)

## Installation

You can install directly from *crates.io* by running `cargo install customasm`.
Then the `customasm` application should automatically become available in your
command-line environment.

You can also download pre-built executables from the
[Releases section](https://github.com/hlorenzi/customasm/releases).

You can compile from source yourself by first cloning the repository and
then simply running `cargo build`.
There's also a battery of tests available at `cargo test`.

## Example

Given the following file:

```asm
#ruledef
{
    load r1, {value: i8} => 0x11 @ value
    load r2, {value: i8} => 0x12 @ value
    load r3, {value: i8} => 0x13 @ value
    add  r1, r2          => 0x21
    sub  r3, {value: i8} => 0x33 @ value
    jnz  {address: u16}  => 0x40 @ address
    ret                  => 0x50
}

multiply3x4:
    load r1, 0
    load r2, 3
    load r3, 4
    
    .loop:
        add r1, r2
        sub r3, 1
        jnz .loop
    
    ret
```

...the assembler will use the `#ruledef` directive to convert the
instructions into binary code:

```asm
 outp | addr | data (base 16)

  0:0 |    0 |          ; multiply3x4:
  0:0 |    0 | 11 00    ; load r1, 0
  2:0 |    2 | 12 03    ; load r2, 3
  4:0 |    4 | 13 04    ; load r3, 4
  6:0 |    6 |          ; .loop:
  6:0 |    6 | 21       ; add r1, r2
  7:0 |    7 | 33 01    ; sub r3, 1
  9:0 |    9 | 40 00 06 ; jnz .loop
  c:0 |    c | 50       ; ret
```