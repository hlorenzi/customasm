# cpudef Directive

This directive controls settings for the target machine, and
defines mnemonics for its instruction set.

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
```

## Configurations

The syntax first expects a list of configuration directives, one per line.
The currently available configuration is:

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

Rules are written as `pattern -> production` and must be separated
by line breaks.

### Pattern

The pattern part of a rule defines its mnemonic and/or parameter slots.

The pattern is a sequence of tokens:  
- For mnemonics, text, or punctuation: just write them out verbatim.
- For parameter slots: write them as `{x}`, with `x` being any valid name.

### Production

The production part of a rule defines its binary representation.
It consists of a single expression.  
The binary representation must have a well-known number of bits,
so that byte alignment can be verified.  
- For literals (like fixed opcodes), use explicitly-sized literals:
the number of bits, followed by a single quote, followed by the value, like `8'0x05`.
- For parameter values, use a bit slice:
the parameter name followed by two numbers inside brackets, like `abc[y:x]`.
`x` and `y` define the rightmost and the leftmost 0-based bit index
of the value that will be selected, counting from the least significant bit.
For example, if `abc = 0xbbaa`, then `abc[7:0] = 0xaa` and `abc[15:8] = 0xbb`.
- Use the concatenation operator `@` to string sub-expressions together, like
`8'0x1a @ addr[15:0]`. All arguments to the concatenation operator must have a
well-known number of bits.
- More complex expressions can also be evaluated; just end them off with a
bit slice if well-known sizes are needed, like `(abc + 0xff)[7:0] @ (pc >> 2)[15:0]`.
- [Predefined variables](#predefined-variables) are also available.

### Constraints

Rules can be given constraints that are used to validate arguments to an
instruction. They come before the `->` token, and each one is marked
with a `::` token. They consist of a condition expression and an
optional description. When the condition is violated, an error is produced
at assembly time.  

For example, we can check whether an argument fits the instruction's
binary representation:
- `jmp {addr} :: addr <= 0xffff -> 8'0x10 @ addr[15:0]`, or
- `jmp {addr} :: addr <= 0xffff, "address out of bounds" -> 8'0x10 @ addr[15:0]`

### Rule Cascading

For the purposes of automatically selecting the best binary
representation for a given instruction (e.g. when there are short
forms for commonly-used arguments), we can use rule cascading.
When we define multiple rules with the same pattern, they are
eligible for cascading. The assembler will select the first
rule (in order of definition) that can have its constraints satisfied.

For example, we can write:

```
#align 8

mov {value} :: value <=     0xff -> 8'0x10 @ value[ 7:0]
mov {value} :: value <=   0xffff -> 8'0x11 @ value[15:0]
mov {value} :: value <= 0xffffff -> 8'0x12 @ value[23:0]
```

This will select the best fitting representation according to
the given argument. If a rule constraint cannot be resolved
due to a label that is not yet defined, the rule is skipped as
if its constraint was violated.

Since it is impossible to force the use of a certain cascading
rule, it is recommended to specify unambiguous rules for all
forms, like:

```
#align 8

mov.b {value} :: value <=     0xff -> 8'0x10 @ value[ 7:0]
mov.w {value} :: value <=   0xffff -> 8'0x11 @ value[15:0]
mov.t {value} :: value <= 0xffffff -> 8'0x12 @ value[23:0]

mov {value} :: value <=     0xff -> 8'0x10 @ value[ 7:0]
mov {value} :: value <=   0xffff -> 8'0x11 @ value[15:0]
mov {value} :: value <= 0xffffff -> 8'0x12 @ value[23:0]
```

### Predefined Variables

The following predefined variables can be used as either arguments to
instructions, or in rule production expressions:
- `pc`  
The address of the current instruction, or, in other words, the
value of the program counter when it reaches the current instruction.

### Rule Examples

With `#align 8`:

Rule | Used as | Output
-----|---------|--------
```load {x} -> 8'0x55 @ x[7:0]``` | ```load 0xff``` | ```0x55 0xff```
```load #{x} -> 8'0x55 @ x[7:0]``` | ```load #0xff``` | ```0x55 0xff```
```load.b {x} -> 8'0x55 @ x[7:0]``` | ```load.b 0xff``` | ```0x55 0xff```
```mov {a} -> 8'0x77 @ a[7:0]``` | ```mov 0xff``` | ```0x77 0xff```
```mov {a} -> 8'0x77 @ a[15:0]``` | ```mov 0xff``` | ```0x77 0x00 0xff```
```mov {a} -> 8'0x77 @ a[15:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 8'0x77 @ a[15:8]``` | ```mov 0x1234``` | ```0x77 0x12```
```mov {a} -> 8'0x77 @ a[15:8] @ a[7:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 8'0x77 @ a[7:0] @ a[15:8]``` | ```mov 0x1234``` | ```0x77 0x34 0x12```
```jmp {a} -> 8'0x99 @ (a + 2)[7:0]``` | ```jmp 0x12``` | ```0x99 0x14```

## Full Examples

```
#cpudef
{
    #align 8
    
    ; we can write the entire rule in one line:
    
    ld  r1, {value} :: value <=     0xff -> 8'0x10 @ value[ 7:0]
    ld  r1, {value} :: value <=   0xffff -> 8'0x11 @ value[15:0]
    ld  r1, {value} :: value <= 0xffffff -> 8'0x12 @ value[23:0]
    
    ; but we can also add in line breaks for readability:
    
    add r1, {value}
        -> 8'0x20 @ value[7:0]
    
    add {addr}, {value}
        -> 8'0x21 @ address[23:0] @ value[7:0]
    
    inc r1
        -> 8'0x30
    
    jmp {addr}
        :: addr <= 0xffffff, "address is out of bounds"
        :: addr >= 0, "address is out of bounds"
        -> 8'0x40 @ address[23:0]
}
```

```
#cpudef
{
	; you can have unusual counts of bits-per-byte too!
    #align 3
    
    lda #{value} -> 3'0b001 @ value[2:0]
    ldx #{value} -> 3'0b010 @ value[2:0]
    sta  {addr}  -> 3'0b011 @ addr[5:0]
    nop          -> 3'0b110
    halt         -> 3'0b111
}
```