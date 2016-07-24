# customasm
This is an assembler that takes custom instruction definitions, and assembles files based on them.

```
Usage:
	customasm [--format=<format>] <def_file> <asm_file> <out_file>
	
Options:
	-f <format>, --format=<format>  The format of the output file. Can be one of: binary, hexdump. [Default: binary]
```

The idea is that, given this definition file:

```
.align 8
.address 24

lda #{a: u8} -> 8'0x01 a;
lda #{a: u16} -> 8'0x02 a;
lda #{a: u24} -> 8'0x03 a;
bra {a: u24} -> 8'0x04 a;
```

...the assembler would take this file:

```
start:
	lda #0xff
	bra loop

loop:
	lda #0xabc
	bra start
```

...and convert it into a binary file with the following contents:

```
0x01 0xff
0x04 0x00 0x00 0x06
0x02 0x0a 0xbc
0x04 0x00 0x00 0x00
```
