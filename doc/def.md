# Definition File Format

This file controls settings for the target machine, and
defines mnemonics for its instruction set.

## Directives

The file starts with a list of configuration directives, one per line.
The currently available directives are:

- `#align <bit_num>`  
Sets the number of bits in a byte for the target machine.  
For example, `#align 8` is the usual configuration for
most modern CPUs.  
Memory addresses are counted in bytes, so, with 8-bit bytes,
address 0x01 actually refers to the bits 8 through 15 in
memory, inclusive.  
Machine instructions must be aligned to a byte boundary,
hence the directive's name. So, with 8-bit bytes, valid
instruction sizes are 8 bits, 16 bits, 24 bits, and so on.

## Rules

The first line not starting with a `#` begins the list of rules.  
A rule defines a valid mnemonic for the target machine, and its
respective binary representation.
Rules are written as `pattern -> production`, one per line.

### Pattern

The pattern part of a rule defines its mnemonic. It may consist of
text, punctuation, and/or argument expressions (that will be
specified by the programmer when invoking the mnemonic).  
The pattern is written as a sequence of tokens separated by spaces.  
- For text and punctuation, just write it out verbatim.
- For argument expressions, write it as `{x}`, with `x`
substituted for any other desired name. If there is more than one
argument, give each one a unique name. This name will be used
in the rule's binary representation to refer to its value.
- Arguments can be given a constraint that, if not
satisfied, will produce an error and abort assembly. Specify it
by adding a colon followed by the constraint after the argument
name, like `{x: constraint}`. Use `_` for the argument's
value, and make sure the constraint expression returns a boolean,
like `{x: _ >= 0 && _ <= 0xff}`. You may use [predefined
variables](#predefined-variables) in the constraint expression.

### Production

The production part of a rule defines its binary representation.
It consists of a sequence of expressions separated by spaces.  
The binary representation must have a fixed number of bits.  
- For literals (like fixed opcodes), use explicitly-sized literals:
the size in bits, followed by a single quote, followed by the value, like `8'0x05`.
- For argument values, use a bit slice:
the argument name followed by two numbers inside brackets, like `abc[y:x]`.
`x` and `y` define the rightmost and the leftmost 0-based bit index
of the value that will be selected, counting from the least significant bit.
For example, if `abc = 0xbbaa`, then `abc[7:0] = 0xaa` and `abc[15:8] = 0xbb`.
- More complex expressions can also be evaluated; just end it off with an
explicit bit slice, like `(abc + 0xff)[7:0]`.
- You may use [predefined variables](#predefined-variables) in expressions.

### Predefined Variables

The following predefined variables can be used in either argument constraints
or production expressions:
- `pc`  
The address of the current instruction, or, in other words, the
value of the program counter when it reaches the current instruction.
Use it like `{x: _ + pc <= 0xff}` or `(x - pc + 1)[7:0]`.

### Rule Cascading

For the purposes of automatically selecting the best binary
representation for a given mnemonic (e.g. when there are short
forms for commonly used arguments), one can use rule cascading.
Write an exclamation mark after the argument name to indicate
that the rule will only be selected if the argument satisfies
the constraint; otherwise, the next rules will be tried instead.
For example, one can write:

```
#align 8

mov {value!: _ <=     0xff} -> 8'0x10 value[ 7:0]
mov {value!: _ <=   0xffff} -> 8'0x11 value[15:0]
mov {value : _ <= 0xffffff} -> 8'0x12 value[23:0]
```

This will select the best fitting representation according to
the argument value. The last rule has no exclamation mark and
thus it is selected when the previous rules fail, or if the
argument value cannot be determined immediately (e.g. when
a label that will only be defined later is used).  
Since it is impossible to force the use of a certain cascading
rule, it is recommended to specify unambiguous rules for all
forms, like:

```
#align 8

mov.b {value: _ <=     0xff} -> 8'0x10 value[ 7:0]
mov.w {value: _ <=   0xffff} -> 8'0x11 value[15:0]
mov.t {value: _ <= 0xffffff} -> 8'0x12 value[23:0]

mov {value!: _ <=     0xff} -> 8'0x10 value[ 7:0]
mov {value!: _ <=   0xffff} -> 8'0x11 value[15:0]
mov {value : _ <= 0xffffff} -> 8'0x12 value[23:0]
```

### Rule Examples

With `#align 8`:

Rule | Used as | Output
-----|---------|--------
```load {x} -> 8'0x55 x[7:0]``` | ```load 0xff``` | ```0x55 0xff```
```load #{x} -> 8'0x55 x[7:0]``` | ```load #0xff``` | ```0x55 0xff```
```load.b {x} -> 8'0x55 x[7:0]``` | ```load.b 0xff``` | ```0x55 0xff```
```mov {a} -> 8'0x77 a[7:0]``` | ```mov 0xff``` | ```0x77 0xff```
```mov {a} -> 8'0x77 a[15:0]``` | ```mov 0xff``` | ```0x77 0x00 0xff```
```mov {a} -> 8'0x77 a[15:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 8'0x77 a[15:8]``` | ```mov 0x1234``` | ```0x77 0x12```
```mov {a} -> 8'0x77 a[15:8] a[7:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 8'0x77 a[7:0] a[15:8]``` | ```mov 0x1234``` | ```0x77 0x34 0x12```
```jmp {a} -> 8'0x99 (a + 2)[7:0]``` | ```jmp 0x12``` | ```0x99 0x14```

## Full Examples

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

```
#align 8

ld  r1, {value!: _ <=     0xff}                   -> 8'0x10 value[ 7:0]
ld  r1, {value!: _ <=   0xffff}                   -> 8'0x11 value[15:0]
ld  r1, {value : _ <= 0xffffff}                   -> 8'0x12 value[23:0]
add r1, {value : _ <=     0xff}                   -> 8'0x20 value[7:0]
add {address: _ <= 0xffffff}, {value : _ <= 0xff} -> 8'0x21 address[23:0] value[7:0]
inc r1                                            -> 8'0x30
jmp {address: _ <= 0xffffff}                      -> 8'0x40 address[23:0]
```

```
#align 3

lda #{value: _ <= 0b111}      -> 3'0b001 value[2:0]
ldx #{value: _ <= 0b111}      -> 3'0b010 value[2:0]
sta  {address: _ <= 0b111111} -> 3'0b011 address[5:0]
nop                           -> 3'0b110
halt                          -> 3'0b111
```