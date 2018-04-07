# Expressions

## Literals

- `123`, `0b111010`, `0o137`, `0x4bf86`  
Integer literals. Bit-widths are derived automatically from used
radix and digits (except decimal literals).

## Operators

The following operators are listed in the order of the lowest precedence
to the highest.

- `=` Assignment
- `@` Concatenation
- `||` Lazy Or
- `&&` Lazy And
- `==`, `!=`, `<`, `<=`, `>`, `>=` Relational
- `|` Binary Or
- `^` Binary Xor
- `&` Binary And
- `<<`, `>>` Binary Shifts
- `+`, `-` Addition and Subtraction
- `*`, `/` Multiplication and Division
- `[hi:lo]` Bit-slice
- `!`, `-` Unary Not and Unary Negation

## Predefined Variables

- `pc`  
The address of the current instruction or the current expression in a
data directive.

## Predefined Functions

- `assert(condition)`  
Generates an error when `condition` is false. Useful to check for
the validity of instruction arguments, and also for instruction cascading.