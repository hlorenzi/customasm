# Usage & Syntax

Keep in mind that indentation is always disregarded, and
is used only for clarity.

## CPU Definition

The typical source file contains a `#cpudef` directive and then
a list of instructions to be assembled for the target machine.

The syntax for the `#cpudef` directive is described at
[cpudef Directive](/doc/cpudef.md).

As an example, the file:

```asm
#cpudef
{
    #align 8
    
    lda {value} -> 0x10 @ value[7:0]
    add {value} -> 0xad @ value[7:0]
    jmp {addr}  -> 0x55 @ addr[15:0]
    inc {addr}  -> 0xcc @ addr[15:0]
    ret         -> 0xee
}

lda 0x77
add 0x01
ret
```

...would be assembled into:

```
0x0000: 10 77
0x0002: ad 01
0x0004: ee
```

We can also use more complex expressions as arguments,
like so (henceforth omitting the preceding `#cpudef` directive
for clarity):

```asm
lda 0x66 + 0x11
add 0x10 - (2 * 4 + 0x07)
ret
```

Even still, we can use predefined variables in argument
expressions. `pc` is the current instruction's address, so
it can be used as:

```asm
inc pc
inc pc
inc pc + 1
```

...and it would be assembled into:

```
0x0000: cc 00 00
0x0002: cc 00 02
0x0004: cc 00 05
```

[Check out the expression documentation](/doc/expr.md) for a full list
of what is available.

## Comments

There are currently only single-line comments. Everything
after a semicolon is treated as a comment and is ignored
by the assembler, until the next line break. For example:

```asm
; load some values
lda 0x77
lda 0x88
lda 0x99 ; load A with 0x99

; disable the next instructions for now:
; lda 0xaa
; lda 0xbb
; lda 0xcc
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

```asm
loop:
    add 0x01
    jmp loop
```

...and have it assembled into:

```
0x0000: 10 77
0x0002: ad 01
0x0004: 55 00 02
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

```asm
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
0x0000: 10 77
0x0002: 55 00 02
0x0005: 10 88
0x0007: 55 00 07
```

The first `jmp .do_it` instruction used the first `.do_it` label as its target.
Likewise, the second `jmp .do_it` instruction used the last `.do_it` label,
because that's the only `.do_it` label that it can see.

## Constants

Numerical constants can also be given a name. The syntax is
`name = value`, followed by a line break.
The value can use complex expressions and
even reference constants that were defined before. For example:

```asm
myvar1 = 0x77
myvar2 = myvar1 + 0x11

lda myvar1
```

There are also local constants, that are defined using a dot before their
names, and can be used just like Local Labels:

```asm
start:
.value = 0xab
    lda .value

loop:
.value = 0xcd
    lda .value
```

## Banks

Up until now, every source file was processed by the assembler as instructions
residing at addresses beginning at `0x0000`. With banks, we can set the
starting program counter value and also configure how the resulting bits
are placed into the output file.

First, define one or more banks as follows:

```asm
#bankdef "mybank"
{
    #addr 0x8000
    #size 0x4000
    #outp 0x10
}
```

This specifies that the bank named `mybank` starts at program counter `0x8000` and
can hold up to `0x4000` bytes. Also, it specifies that these bytes should be
placed starting at position `0x10` in the output file (as in, `0x10` bytes from the
beginning of the file). This directive automatically switches to assembling at
the newly defined bank, but if you define more banks, you can switch between them
with the following:

```asm
#bank "mybank"
```

When you switch banks, the assembler resumes from where it left off the program
counter. You can interleave bank assembling in this way.

### Non-Writable Banks

If you define a bank without an `#outp` attribute, it will be treated as non-writable:
you won't be able to write data to it, only allocate space (e.g. through the `#res`
directive). It also won't take up space in the output file.

### Fill Attribute

If you define a bank with a `#fill` attribute such as the following:

```asm
#bankdef "mybank"
{
    #addr 0x8000
    #size 0x4000
    #outp 0x10
    #fill
}
```

...then it will occupy the entire size indicated in the output file. Usually, without
this attribute, the assembler will truncate the output file if no more data lies
beyond a certain point (and if the bank is the last bank in the output file).


### Address Directive

You can skip ahead to a certain address within the current bank by using
this directive. Skipped bits will be filled with zeroes. For example:

```asm
#d8 0xab, 0xcd, 0xef
#addr 0x8
#d8 0xab, 0xcd, 0xef
```

...would be assembled into:

```
0x0000: ab cd ef
0x0003: 00 00 00 00 00
0x0008: ab cd ef
```

## Data Directive

This directive copies a sequence of values verbatim to the output. Its
name contains the bit-size of each component in the sequence. This
bit-size can be any value, as long as the final address is left aligned
to the machine's byte boundaries. For example:

```asm
lda 0x77
#d4 0x1, 0x2, 0x3, 0x4
#d8 0x12, 0x34, 0x56, 0x78
#d16 0x1234, 0x5678
#d32 0x1234, 0x5678
```

...would be assembled into:

```
0x0000: 10 77
0x0002: 12 34
0x0004: 12 34 56 78
0x0008: 12 34 56 78
0x000c: 00 00 12 34 00 00 56 78
```

Note that the `#d32` directive's arguments, `0x1234, 0x5678`, were
sign-extended to match the directive's bit-size.

## String Directive

This directive copies the UTF-8 representation of a string to
the output. Rust-like escape sequences and Unicode characters are available.
For example:

```asm
#str "abcd"
#str "\n\r\0"
#str "\x12\x34"
#str "木"
```

...would be assembled into:

```
0x0000: 61 62 63 64
0x0004: 0a 0d 00
0x0007: 12 34
0x0009: e6 9c a8
```

If the string's length is needed, we can use a bit of arithmetic
to derive it:

```asm
helloworld:
    #str "Hello, world!\0"
    
helloworldLen = pc - helloworld
```

## Align Directive

This directive advances the current address until its value is a multiple
of the given value, but does nothing if it already is.

```asm
#d8 0xff
    
#align 4
loop:
    jmp loop
```

...would be assembled to:

```
0x0000: ff 00 00 00
0x0004: 55 00 04
```

## Reserve Directive

This directive advances the current address by
the given number of bytes, effectively reserving a location for any
other desired purpose. For example, in a machine where data and
instructions reside on the same memory space, we could do:

```asm
    jmp start
  
variable:
    #res 1

start:
    lda 0x77
    inc variable
```

...and it would be assembled into:

```
0x0000: 55 00 04
0x0003: 00
0x0004: 10 77
0x0006: cc 00 03
```

## Include Directives

These directives include external data from other files into
the output. All filenames are relative to the current Source
file being assembled. The files can also be located inside
subfolders.

### Include Source Directive

This directive effectively copies the given file's content as
source code, merging it into the current file being assembled.
For example, suppose this was the main Source file:

```asm
start:
    lda 0x77
  
#include "extra.asm"
```

...and that there were another file named `extra.asm` in the
same directory, with the following contents:

```asm
jmp start
```

The files are effectively merged together. The `jmp start` in
the `extra.asm` file can naturally see the label defined on the
main file. This would be the output:

```
0x0000: 10 77
0x0002: 55 00 00
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

```asm
lda 0x77
#incbin "text.bin"
```

...and given the following `text.bin` file:

```
hello
```

...everything would be assembled into:

```
0x0000: 10 77
0x0002: 68 65 6c 6c 6f
```

### Include Binary String Directive

This directive interprets the contents of the given file as
a string of binary digits, and copies that to the output, verbatim.
For example, given the following Source file:

```asm
lda 0x77
#incbinstr "data.txt"
```

...and given the following `data.txt` file:

```
01011010
```

...everything would be assembled into:

```
0x0000: 10 77
0x0002: 5a
```

This is specially useful when used in conjunction with
customasm's `binstr` output format.

### Include Hexadecimal String Directive

This directive interprets the contents of the given file as
a string of hexadecimal digits, and copies that to the output,
verbatim. For example, given the following Source file:

```asm
lda 0x77
#inchexstr "data.txt"
```

...and given the following `data.txt` file:

```
5affc068
```

...everything would be assembled into:

```
0x0000: 10 77
0x0002: 5a ff c0 68
```

This is specially useful when used in conjunction with
customasm's `hexstr` output format.