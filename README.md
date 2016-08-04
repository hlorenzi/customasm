# customasm
This is an assembler that takes custom machine instruction definitions, and assembles files based on them.  
Check out the wiki for usage instructions.

```
Usage:
	customasm [options] <def_file> <asm_file> [<out_file>]
	customasm -v | --version
	customasm -h | --help
	
Options:
	-q, --quiet                     Do not print progress to stdout.
	-f <format>, --format=<format>  The format of the output file. Can be one of:
	                                    binary, binstr, hexstr, bindump, hexdump.
	                                    [default: hexdump]
	-v, --version                   Display version information.
	-h, --help                      Display help.
```

The idea is that, given this definition file:

```
.align 8
.address 16

load r1, {value: u8} -> 8'0x11 value[7:0]
load r2, {value: u8} -> 8'0x12 value[7:0]
load r3, {value: u8} -> 8'0x13 value[7:0]
add  r1, r2          -> 8'0x21
sub  r3, {value: u8} -> 8'0x33 value[7:0]
jnz  {address: u16}  -> 8'0x40 address[15:0]
ret                  -> 8'0x50
```

...the assembler would take this file:

```
.address 0x8000

multiply3x4:
	load r1, 0
	load r2, 3
	load r3, 4
	
	'loop:
		add r1, r2
		sub r3, 1
		jnz 'loop
	
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
