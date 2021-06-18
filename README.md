# customasm
This is an assembler that takes custom, user-defined instruction sets
and uses them to assemble source files.  
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

[🖥️ Try it right now on your browser!](https://hlorenzi.github.io/customasm/web/)

[🕹️ Check out an example project](/examples/nes/) which targets the NES!

[📜 Check out the changelog](/CHANGELOG.md) for new features!

[⌨️ Install the VSCode syntax highlight extension!](https://marketplace.visualstudio.com/items?itemName=hlorenzi.customasm-vscode)

[❤️ Support me!](https://accounts.hlorenzi.com/supporters)

## Documentation

[📚 Check out the Documentation](https://github.com/hlorenzi/customasm/wiki/User-Guide)
for advanced features and a how-to guide!

## Installation

You can install directly from crates.io by running `cargo install customasm`.
Then the `customasm` application should automatically become available in your
command-line environment.

You can also download pre-built executables from the
[Releases section](https://github.com/hlorenzi/customasm/releases).

You can compile from source yourself by first cloning the repository and
then simply running `cargo build`.
There's also a battery of tests available at `cargo test`.

## Upgrade to v0.11

[📖 Check out instructions for migration from older versions to v0.11!](https://github.com/hlorenzi/customasm/wiki/Migrating-to-v0.11)

## Example

Given the following file:

```asm
#ruledef
{
    load r1, {value} => 0x11 @ value`8
    load r2, {value} => 0x12 @ value`8
    load r3, {value} => 0x13 @ value`8
    add  r1, r2      => 0x21
    sub  r3, {value} => 0x33 @ value`8
    jnz  {address}   => 0x40 @ address`16
    ret              => 0x50
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
 outp | addr | data

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

## Command-Line Usage

```
Usage: customasm [options] <asm-file-1> ... <asm-file-N>

Options:
    -f, --format FORMAT The format of the output file. Possible formats:
                        binary, annotated, annotatedbin, binstr, hexstr,
                        bindump, hexdump, mif, intelhex, deccomma, hexcomma,
                        decc, hexc, logisim8, logisim16
    -o, --output [FILE] The name of the output file.
    -s, --symbol [FILE] The name of the output symbol file.
    -t, --iter [NUM]    The max number of passes the assembler will attempt
                        (default: 10).
    -p, --print         Print output to stdout instead of writing to a file.
    -q, --quiet         Suppress progress reports.
    -v, --version       Display version information.
    -h, --help          Display this information.
```
