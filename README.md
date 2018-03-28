# customasm
This is an assembler that takes custom instruction set definitions
and uses them to assemble source files.  
This can be useful if you'd like to test out a new virtual machine's bytecode,
or even if you're eager to write programs for that new processor architecture 
you just implemented in FPGA!

[Check out the Releases section](https://github.com/hlorenzi/customasm/releases) 
for pre-built binaries.  

[Check out the documentation](/doc/index.md) for user instructions.

You can compile from source by simply doing `cargo build`. There's also a
battery of tests available at `cargo test`.

Usage of the command line interface is as follows:

```
Usage: customasm [options] <asm-file>

Options:
    -h, --help          Display this information.
    -v, --version       Display version information.
    -q, --quiet         Suppress progress reports.
    -f, --format FORMAT The format of the output file. Possible formats:
                        binary, binstr, hexstr, bindump, hexdump
    -o, --output FILE   The name of the output file.
    -p, --print         Print output to stdout instead of writing to a file.
```

As an example, given the following file:

```asm
#cpudef
{
    #align 8
    
    load r1, {value} -> 8'0x11 @ value[7:0]
    load r2, {value} -> 8'0x12 @ value[7:0]
    load r3, {value} -> 8'0x13 @ value[7:0]
    add  r1, r2      -> 8'0x21
    sub  r3, {value} -> 8'0x33 @ value[7:0]
    jnz  {address}   -> 8'0x40 @ address[15:0]
    ret              -> 8'0x50
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
