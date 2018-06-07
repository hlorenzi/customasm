# customasm
This is an assembler that takes custom instruction set definitions
and uses them to assemble source files.  
This can be useful if you'd like to test out a new virtual machine's bytecode,
or even if you're eager to write programs for that new processor architecture 
you just implemented in FPGA!

[Try it right now on your browser!](https://hlorenzi.github.io/customasm/webasm/)

[Check out the Releases section](https://github.com/hlorenzi/customasm/releases) 
for pre-built binaries.  

[Check out the documentation](/doc/index.md) for user instructions.

Also, [check out an example project](/examples/nes/) which targets the NES!

You can compile from source by simply doing `cargo build`. There's also a
battery of tests available at `cargo test`.

## Command Line Usage

```
Usage: customasm [options] <asm-file-1> ... <asm-file-N>

Options:
    -f, --format FORMAT The format of the output file. Possible formats:
                        binary, binstr, hexstr, bindump, hexdump
    -i, --include FILE  Specifies an additional file for processing before the
                        given <asm-files>.
    -o, --output FILE   The name of the output file.
    -p, --print         Print output to stdout instead of writing to a file.
    -q, --quiet         Suppress progress reports.
    -v, --version       Display version information.
    -h, --help          Display this information.
```

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

```
0x0100: 11 00
0x0102: 12 03
0x0104: 13 04
0x0106: 21
0x0107: 33 01
0x0109: 40 01 06
0x010c: 50
```
