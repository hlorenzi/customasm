# customasm
This is an assembler that takes custom instruction set definitions
and uses them to assemble source files.  
This can be useful if you'd like to test out a new virtual machine's bytecode,
or even if you're eager to write programs for that new processor architecture 
you just implemented in FPGA!

[![Latest Release][badge-latest-img]][badge-latest-url]
[![Releases][badge-downloads-img]][badge-downloads-url]
[![Discord][badge-discord-img]][badge-discord-url]

[badge-latest-img]: https://img.shields.io/github/v/release/hlorenzi/customasm
[badge-latest-url]: https://github.com/hlorenzi/customasm/releases

[badge-downloads-img]: https://img.shields.io/github/downloads/hlorenzi/customasm/total
[badge-downloads-url]: https://github.com/hlorenzi/customasm/releases

[badge-discord-img]: https://img.shields.io/discord/394999035540275222?label=Discord&logo=discord
[badge-discord-url]: https://discord.gg/pXeDXGD

[📱 Try it right now in your browser!](https://hlorenzi.github.io/customasm/web/)
 
[🎁 Check out the Releases section](https://github.com/hlorenzi/customasm/releases) 
for pre-built binaries.

[📖 Check out the User Guide](https://github.com/hlorenzi/customasm/wiki/User-Guide)
on how to use the main features!

[📋 Check out the documentation](/doc/index.md) for more in-depth instructions.

[🕹 Check out an example project](/examples/nes/) which targets the NES!

You can compile from source by simply doing `cargo build`. There's also a
battery of tests available at `cargo test`.

## Example

Given the following file:

```asm
#cpudef
{
    #bits 8
    
    load r1, {value} -> 0x11 @ value[7:0]
    load r2, {value} -> 0x12 @ value[7:0]
    load r3, {value} -> 0x13 @ value[7:0]
    add  r1, r2      -> 0x21
    sub  r3, {value} -> 0x33 @ value[7:0]
    jnz  {address}   -> 0x40 @ address[15:0]
    ret              -> 0x50
}

#addr 0x100

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

...the assembler would use the `#cpudef` rules to convert the instructions into binary code:

```asm
  outp | addr | data

 100:0 |  100 |          ; multiply3x4:
 100:0 |  100 | 11 00    ; load r1, 0
 102:0 |  102 | 12 03    ; load r2, 3
 104:0 |  104 | 13 04    ; load r3, 4
 106:0 |  106 |          ; .loop:
 106:0 |  106 | 21       ; add r1, r2
 107:0 |  107 | 33 01    ; sub r3, 1
 109:0 |  109 | 40 01 06 ; jnz .loop
 10c:0 |  10c | 50       ; ret
```

## Command Line Usage

```
Usage: customasm [options] <asm-file-1> ... <asm-file-N>

Options:
    -f, --format FORMAT The format of the output file. Possible formats:
                        binary, annotated, annotatedbin, binstr, hexstr,
                        bindump, hexdump, mif, intelhex, deccomma, hexcomma,
                        decc, hexc, logisim8, logisim16
                        
    -o, --output FILE   The name of the output file.
    -s, --symbol FILE   The name of the output symbol file.
    -p, --print         Print output to stdout instead of writing to a file.
    -q, --quiet         Suppress progress reports.
    -v, --version       Display version information.
    -h, --help          Display this information.
```
