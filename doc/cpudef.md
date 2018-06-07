# cpudef Directive

This directive controls settings for the target machine, and
defines mnemonics for its instruction set.

```asm
#cpudef
{
    #bits 8
    
    lda {value} -> 0x10 @ value[7:0]
    add {value} -> 0xad @ value[7:0]
    jmp {addr}  -> 0x55 @ addr[15:0]
    inc {addr}  -> 0xcc @ addr[15:0]
    ret         -> 0xee
}
```

## Configurations

The syntax first expects a list of configuration directives, one per line.
The currently available configurations are:

- `#bits <num>`  
Sets the number of bits-per-byte for the target machine.  
For example, `#bits 8` is the usual configuration for
most modern CPUs.  
Memory addresses are counted in bytes, so, with 8-bit bytes,
address 0x01 refers to the bits 8 through 15 in
memory, 0-indexed, inclusive.  
The size of each instruction's binary representation must be
some multiple of the byte size. So, with 8-bit bytes, valid
instruction sizes are 8 bits, 16 bits, 24 bits, and so on.

- `#labelalign <value>`  
Whenever a label is defined in the source code, the assembler will
align the current address to be a multiple of this value.  
Equivalent to using `#align` before every label.

- `#tokendef <name>`  
Creates a group of tokens with associated values, which can
be used in place of arguments (e.g. for named registers).  
See below for usage in parameters, and check the example at
the bottom of the page. Syntax is as follows:
```asm
#tokendef reg
{
    a = 1
    b = 2
    c = 3
}
```

## Rules

The first line not starting with a `#` begins the list of rules.  
A rule defines a valid mnemonic for the target machine, and its
respective binary representation.

Rules are written as `pattern -> output` and must be separated
by line breaks.

### Pattern

The pattern part of a rule defines its mnemonic and/or parameter slots.

The pattern is a sequence of tokens:  
- For mnemonics, text, or punctuation: just write them out verbatim.
- For parameter slots: write them as `{x}`, with `x` being any valid name.
- For custom token groups declared with `#tokendef`, write them as `{x: name}`,
with `name` being the name given at the `#tokendef` declaration (`reg` in the
example above).

### Output

The output part of a rule defines its binary representation.
It consists of a single expression, whose integer result is
sent to the output. Note is that this integer result
must have a known width (number of bits) to allow for advancing
the current address while assembling, and to allow for verification
to byte alignment.

- For literals (like fixed opcodes), like `0x7f`, the width is derived automatically
from the radix and number of digits used. If it's necessary to explicitly state the
width, use explicitly-sized literals:
the number of bits, followed by a single quote, followed by the value, like `8'0x05`.
- For any other value, including arguments to the instruction, use a bit slice:
the expression followed by two numbers inside brackets, like `abc[hi:lo]`.
`hi` and `lo` define the rightmost and the leftmost 0-based bit index
of the value that will be selected, counting from the least significant bit.
For example, if `abc = 0xbbaa`, then `abc[7:0] = 0xaa` and `abc[15:8] = 0xbb`.
- More complex expressions can also be evaluated; just end them off with a
bit slice if known widths are required, like `(abc + 0xff)[7:0] @ (pc >> 2)[15:0]`.
- Use the concatenation operator `@` to string sub-expressions together, like
`0x1a @ addr[15:0]`. All arguments to the concatenation operator must have a
known width.
- [Check out the expression documentation](/doc/expr.md) for a full list of
what is available.

An expression can also be a block, the result of which is defined to be its
last sub-expression. Blocks are especially useful to evaluate more complex
expressions in separate steps (by using local variables), or to create
assertions (e.g. that verify the validity of passed arguments). For example,
the following rule would only allow arguments between `0x00` and `0x10`:

```asm
load {value} ->
{
    assert(value >= 0x00)
    assert(value <  0x10)
    
    0x1 @ value[7:0]
}
```

Sub-expressions in a block can be separated by linebreaks or commas.

Local variables can also be used by assigning a value to any name:

```asm
load {value} ->
{
    finalValue = value * 2
    
    assert(finalValue >= 0x00)
    assert(finalValue <  0x10)
    
    0x1 @ finalValue[7:0]
}
```

### Rule Cascading

For the purposes of automatically selecting the best binary
representation for a given instruction (e.g. when there are short
forms for commonly-used arguments), we can use rule cascading.
When we define multiple rules with the same pattern, they are
eligible for cascading. The assembler will select the first
rule (in order of definition) that can have its expression evaluated
without error (and without any assertion failures).

For example, we can write:

```asm
#bits 8

mov {value} -> { assert(value <=     0xff), 0x10 @ value[ 7:0] }
mov {value} -> { assert(value <=   0xffff), 0x11 @ value[15:0] }
mov {value} -> { assert(value <= 0xffffff), 0x12 @ value[23:0] }
```

If the arguments to the instruction cannot be resolved in the first
pass (e.g. when using a label that will only be defined later), the
last rule in the cascading series is always the one selected.

Since it is impossible to force the use of a certain rule under
cascading, it may be convenient to specify unambiguous rules for all
forms, like:

```asm
#bits 8

mov.b {value} -> { assert(value <=     0xff), 0x10 @ value[ 7:0] }
mov.w {value} -> { assert(value <=   0xffff), 0x11 @ value[15:0] }
mov.t {value} -> { assert(value <= 0xffffff), 0x12 @ value[23:0] }

mov {value} -> { assert(value <=     0xff), 0x10 @ value[ 7:0] }
mov {value} -> { assert(value <=   0xffff), 0x11 @ value[15:0] }
mov {value} -> { assert(value <= 0xffffff), 0x12 @ value[23:0] }
```

### Rule Examples

With `#bits 8`:

Rule | Used as | Output
-----|---------|--------
```load {x} -> 0x55 @ x[7:0]``` | ```load 0xff``` | ```0x55 0xff```
```load #{x} -> 0x55 @ x[7:0]``` | ```load #0xff``` | ```0x55 0xff```
```load.b {x} -> 0x55 @ x[7:0]``` | ```load.b 0xff``` | ```0x55 0xff```
```mov {a} -> 0x77 @ a[7:0]``` | ```mov 0xff``` | ```0x77 0xff```
```mov {a} -> 0x77 @ a[15:0]``` | ```mov 0xff``` | ```0x77 0x00 0xff```
```mov {a} -> 0x77 @ a[15:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 0x77 @ a[15:8]``` | ```mov 0x1234``` | ```0x77 0x12```
```mov {a} -> 0x77 @ a[15:8] @ a[7:0]``` | ```mov 0x1234``` | ```0x77 0x12 0x34```
```mov {a} -> 0x77 @ a[7:0] @ a[15:8]``` | ```mov 0x1234``` | ```0x77 0x34 0x12```
```jmp {a} -> 0x99 @ (a + 2)[7:0]``` | ```jmp 0x12``` | ```0x99 0x14```

## Full Examples

```asm
#cpudef
{
    #bits 8
    
    ; we can write the entire rule in one line:
    
    inc r1 -> 0x30
    
    ld r1, {value} -> { assert(value <=     0xff), 0x10 @ value[ 7:0] }
    ld r1, {value} -> { assert(value <=   0xffff), 0x11 @ value[15:0] }
    ld r1, {value} -> { assert(value <= 0xffffff), 0x12 @ value[23:0] }
    
    ; but we can also add in line breaks for readability:
    
    add r1, {value} ->
        0x20 @ value[7:0]
    
    add {addr}, {value} ->
        0x21 @ address[23:0] @ value[7:0]
    
    jmp {addr} ->
    {
        assert(addr <= 0xffffff)
        assert(addr >= 0)

        0x40 @ address[23:0]
    }
}
```

```asm
#cpudef
{
    ; you can have unusual counts of bits-per-byte too!
	
    #bits 3
    
    lda #{value} -> 0b001 @ value[2:0]
    ldx #{value} -> 0b010 @ value[2:0]
    sta  {addr}  -> 0b011 @ addr[5:0]
    nop          -> 0b110
    halt         -> 0b111
}
```

```asm
#cpudef
{
    ; example with named registers
    
    #bits 8
    
    #tokendef reg
    {
        r0 = 0
        r1 = 1
        r2 = 2
        r3 = 3
    }
    
    mov {dest: reg}, {value} -> 0b111100 @ dest[1:0] @ value[7:0]
}
```