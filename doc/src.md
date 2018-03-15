# Source File Format

These files contain the source code that will be
assembled for the target machine.

## CPU Definition

The simplest source file contains a `#cpudef` directive and then
a list of instructions to be assembled for the target machine.
Indentation is always disregarded everywhere.

The syntax for the `#cpudef` directive is described at
[cpudef Directive](/doc/cpudef.md).

As an example, the file:

```
#cpudef
{
    #align 8
    
    lda {value} -> 8'0x10 @ value[7:0]
    add {value} -> 8'0xad @ value[7:0]
    jmp {addr}  -> 8'0x55 @ addr[15:0]
    inc {addr}  -> 8'0xcc @ addr[15:0]
    ret         -> 8'0xee
}

lda 0x77
add 0x01
ret
```

...would be assembled into:

```
0x10 0x77
0xad 0x01
0xee
```

We can also use more complex expressions as arguments,
like so (henceforth omitting the preceding `#cpudef` directive
for clarity):

```
lda 0x66 + 0x11
add 0x10 - (2 * 4 + 0x07)
ret
```

Even still, we can use predefined variables in argument
expressions. `pc` is the current instruction's address, so
it can be used as:

```
inc pc
inc pc
inc pc + 1
```

...and it would be assembled into:

```
0xcc 0x00 0x00
0xcc 0x00 0x02
0xcc 0x00 0x05
```

## Comments

There are currently only single-line comments. Everything
after a semicolon is treated as a comment and is ignored
by the assembler. For example:

```
; load two values
lda 0x77
lda 0x88
lda 0x99 ; I'm not sure about this one

; ignore the next instruction for now
; lda 0xaa
```

## Labels

The current address can be given a name to allow it to
be referenced, for example, by jump instructions.

### Global Labels

These kinds of labels must be unique throughout the entire
source code. The syntax is `label_name:`.
Again, indentation is disregarded; there is no actual need
to indent instructions more than labels.

Using the previous Instruction Set file, we could write:

```
loop:
    add 0x01
    jmp loop
```

...and have it assembled into:

```
0x10 0x77
0xad 0x01
0x55 0x00 0x02
```

We can see that the `jmp` instruction used the `loop`
label as its target. This was reflected in the output as
`0x55 0x00 0x02`, meaning the `loop` label is pointing
at the address `0x0002`.

Also, there is no need that
the label be already defined when it is referenced by
an instruction; its definition may appear later in
the Source file. Note that, if this is the case, rule
cascading may not work, and the assembler may always
select the instruction rule defined last.

### Local Labels

Local Labels are only visible between the two Global Labels
that they are defined within. The syntax is `.label_name:`.
Multiple Local Labels can
have the same name if they are defined inside different
bodies of Global Labels. For example:

```
start:
    lda 0x77
.do_it:
    jmp .do_it

loop:
    lda 0x88
.do_it:
    jmp .do_it
```

...and have it assembled into:

```
0x10 0x77
0x55 0x00 0x02
0x10 0x88
0x55 0x00 0x07
```

The first `jmp .do_it` instruction used the first `.do_it` label as its target.
Likewise, the second `jmp .do_it` instruction used the last `.do_it` label,
because that's the only `.do_it` label that it can see.

## Constants

Numerical constants can also be given a name. The syntax is
`name = value`, followed by a line break.
The value can use complex expressions and
even reference constants that were defined before. For example:

```
sevenseven = 0x77
eighteight = sevenseven + 0x11

lda sevenseven
```

There are also local constants, that are defined using a dot before their
names, and can be used just like Local Labels:

```
start:
.value = 0xab
    lda .value

loop:
.value = 0xcd
    lda .value
```

## Directives

Directives invoke special behaviors in the assembler.

### Address Directive

Up until now, every source file was seen by the assembler as instructions
residing at addresses beginning at `0x0000`. With the Address directive, we can
change what address the assembler should count from. For example:

```
#addr 0x8000
start:
    lda 0x77
    jmp start

#addr 0xf000
loop:
    add 0x01
.do_it:
    jmp .do_it
```

...would be assembled into:

```
0x10 0x77
0x55 0x80 0x00
0xad 0x01
0x55 0xf0 0x02
```

The address of the instructions has been altered by the directives, but
note that their binary representations are still located at the beginning
of the Output file (and not at `0x8000` bytes into it). Also, the groups
starting at `0x8000` and `0xf000` are still right next to each other,
without any gaps. We can alter this behavior using the following directive.

### Output Directive

This directive alters where in the Output file the next instructions'
binary representations will be placed. For example:

```
#outp 0x4
start:
    lda 0x77
    jmp start
```

...would be assembled into:

```
0x00 0x00 0x00 0x00
0x10 0x77
0x55 0x00 0x00
```

The first instruction has been placed at address `0x4` in the Output
file. But note that this doesn't change the address of the instructions
themselves; the `start` label still points to the address `0x0000`. We
can use both directives to align instruction addresses and output
locations, like so:

```
#addr 0x4
#outp 0x4
start:
    lda 0x77
    jmp start
```

The `start` label would now point to the address `0x0004`, with the
binary representation still being offset by 4 bytes at the beginning.

### Data Directive

This directive copies a sequence of values verbatim to the output. Its
name contains the bit-size of each component in the sequence. This
bit-size can be any value, as long as the final address is left aligned
to the machine's byte boundaries. For example:

```
lda 0x77
#d4 0x1, 0x2, 0x3, 0x4
#d8 0x12, 0x34, 0x56, 0x78
#d16 0x1234, 0x5678
#d32 0x1234, 0x5678
```

...would be assembled into:

```
0x10 0x77
0x12 0x34
0x12 0x34 0x56 0x78
0x12 0x34 0x56 0x78
0x00 0x00 0x12 0x34 0x00 0x00 0x56 0x78
```

Note that the `#d32` directive's arguments, `0x1234, 0x5678`, were
sign-extended to match the directive's bit-size.

### String Directive

This directive copies the UTF-8 representation of a string to
the output. Escape sequences and Unicode characters are available.
For example:

```
#str "abcd"
#str "\n\r\0"
#str "\x12\x34"
#str "木"
```

...would be assembled into:

```
0x61 0x62 0x63 0x64
0x0a 0x0d 0x00
0x12 0x34
0xe6 0x9c 0xa8
```

If the string's length is needed, we can use a bit of arithmetic
to derive it:

```
helloworld:
	#str "Hello, world!\0"
	
helloworld_len = pc - helloworld
```

### Reserve Directive

This directive advances the instruction *and* output addresses by
the given number of bytes, effectively reserving a location for any
other desired purpose. For example, in a machine where data and
instructions reside on the same memory space, we could do:

```
    jmp start
  
variable:
    #res 1

start:
    lda 0x77
    inc variable
```

...and it would be assembled into:

```
0x55 0x00 0x04
0x00
0x10 0x77
0xcc 0x00 0x03
```

### Include Directives

These directives include external data from other files into
the output. All filenames are relative to the current Source
file being assembled. The files can also be located inside
subfolders.

#### Include Source Directive

This directive effectively copies the given file's content as
source code, merging it into the current file being assembled.
For example, suppose this was the main Source file:

```
start:
    lda 0x77
  
#include "extra.asm"
```

...and that there were another file named `extra.asm` in the
same directory, with the following contents:

```
jmp start
```

The files are effectively merged together. The `jmp start` in
the `extra.asm` file can naturally see the label defined on the
main file. This would be the output:

```
0x10 0x77
0x55 0x00 0x00
```

Note that, even though the files are logically merged together, the
assembler still tracks their location on the directory tree. If
you included a file in a subfolder (like `#include "stuff/extra.asm"`),
other include directives inside the `stuff/extra.asm` file would
be resolved relative to the `stuff/` folder.

#### Include Binary Directive

This directive copies the binary contents of the given file verbatim
to the output. Since supported filesystems are 8-bit based, this
directive can only be used on machines with alignments that are
multiples of 8. For example, given the following Source file:

```
lda 0x77
#incbin "text.bin"
```

...and given the following `text.bin` file:

```
hello
```

...everything would be assembled into:

```
0x10 0x77
0x68 0x65 0x6c 0x6c 0x6f
```

#### Include Binary String Directive

This directive interprets the contents of the given file as
a string of binary digits, and copies that to the output, verbatim.
For example, given the following Source file:

```
lda 0x77
#incbinstr "data.txt"
```

...and given the following `data.txt` file:

```
01011010
```

...everything would be assembled into:

```
0x10 0x77
0x5a
```

This is specially useful when used in conjunction with
customasm's `binstr` output format.

#### Include Hexadecimal String Directive

This directive interprets the contents of the given file as
a string of hexadecimal digits, and copies that to the output,
verbatim. For example, given the following Source file:

```
lda 0x77
#inchexstr "data.txt"
```

...and given the following `data.txt` file:

```
5affc068
```

...everything would be assembled into:

```
0x10 0x77
0x5a 0xff 0xc0 0x68
```

This is specially useful when used in conjunction with
customasm's `hexstr` output format.