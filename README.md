# customasm
This is an assembler that takes custom instruction definitions, and assembles files based on them.  
Check out the wiki for instructions.

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
.address 24

load #{a: u8}  -> 8'0x01 a;
load #{a: u16} -> 8'0x02 a;
load #{a: u24} -> 8'0x03 a;
jump  {a: u24} -> 8'0x04 a;
```

...the assembler would take this file:

```
start:
	load #0xff
	jump loop

loop:
	load #0xabc
	jump start
```

...and convert it into a binary file with the following contents:

```
0x01 0xff
0x04 0x00 0x00 0x06
0x02 0x0a 0xbc
0x04 0x00 0x00 0x00
```
