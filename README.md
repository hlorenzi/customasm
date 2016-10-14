# customasm
This is an assembler that takes custom instruction set definitions,
and assembles files based on that.  
This can be useful if you'd like to test out a new virtual machine's bytecode,
or even if you're eager to write programs for the new processor architecture 
you just implemented in FPGA!  
Check out the Releases section for pre-built binaries.  
  
Check out the documentation for usage instructions:
- [Definition File Format](/doc/def.md)

```
Usage: customasm [options] <def-file> <asm-file> [<out-file>]

Options:
    -h, --help          Display this information.
    -v, --version       Display version information.
    -q, --quiet         Suppress progress reports.
    -f, --format FORMAT The format of the output file. Possible formats:
                        binary, binstr, hexstr, bindump, hexdump
```

The idea is that, given this definition file:

```
#align 8

load r1, {value: _ <= 0xff} -> 8'0x11 value[7:0]
load r2, {value: _ <= 0xff} -> 8'0x12 value[7:0]
load r3, {value: _ <= 0xff} -> 8'0x13 value[7:0]
add  r1, r2                 -> 8'0x21
sub  r3, {value: _ <= 0xff} -> 8'0x33 value[7:0]
jnz  {address: _ <= 0xffff} -> 8'0x40 address[15:0]
ret                         -> 8'0x50
```

...the assembler would take this file:

```
#address 0x8000

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

...and convert it into a binary file with the following contents:

```
0x11 0x00
0x12 0x03
0x13 0x04
0x21
0x33 0x01
0x40 0x80 0x06
0x50
```
