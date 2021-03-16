; :::
#subruledef opcode
{
    A => 0x1
    B => 0x2
    C => 0x3
}

#subruledef condition
{
    X  => 0xa
    Y  => 0xb
    Z  => 0xc
    {} => 0xa ; empty pattern, default to X
}

#subruledef instruction
{
    {opc: opcode}{cnd: condition} => opc @ cnd
}

#ruledef
{
    {ins: instruction} {val: u8} => ins @ val
}

A  0x33 ; = 0x1a33
B  0x33 ; = 0x2a33
C  0x33 ; = 0x3a33
AX 0x33 ; = 0x1a33
AY 0x33 ; = 0x1b33
AZ 0x33 ; = 0x1c33
BX 0x33 ; = 0x2a33
BY 0x33 ; = 0x2b33
BZ 0x33 ; = 0x2c33
CX 0x33 ; = 0x3a33
CY 0x33 ; = 0x3b33
CZ 0x33 ; = 0x3c33


; :::
#subruledef opcode
{
    A => 0x1
    B => 0x2
    C => 0x3
}

#subruledef condition
{
    X => 0xa
    Y => 0xb
    Z => 0xc
}

#subruledef instruction
{
    {opc: opcode}                 => opc @ 0xa ; default to X
    {opc: opcode}{cnd: condition} => opc @ cnd
}

#ruledef
{
    {ins: instruction} {val: u8} => ins @ val
}

A  0x33 ; = 0x1a33
B  0x33 ; = 0x2a33
C  0x33 ; = 0x3a33
AX 0x33 ; = 0x1a33
AY 0x33 ; = 0x1b33
AZ 0x33 ; = 0x1c33
BX 0x33 ; = 0x2a33
BY 0x33 ; = 0x2b33
BZ 0x33 ; = 0x2c33
CX 0x33 ; = 0x3a33
CY 0x33 ; = 0x3b33
CZ 0x33 ; = 0x3c33


; :::
#subruledef opcode
{
    A => 0x1
    B => 0x2
    C => 0x3
}

#subruledef condition
{
    X => 0xa
    Y => 0xb
    Z => 0xc
}

#subruledef instruction
{
    {opc: opcode}                  => opc @ 0xa ; default to X
    {opc: opcode}-{cnd: condition} => opc @ cnd
}

#ruledef
{
    {ins: instruction} {val: u8} => ins @ val
}

A   0x33 ; = 0x1a33
B   0x33 ; = 0x2a33
C   0x33 ; = 0x3a33
A-X 0x33 ; = 0x1a33
A-Y 0x33 ; = 0x1b33
A-Z 0x33 ; = 0x1c33
B-X 0x33 ; = 0x2a33
B-Y 0x33 ; = 0x2b33
B-Z 0x33 ; = 0x2c33
C-X 0x33 ; = 0x3a33
C-Y 0x33 ; = 0x3b33
C-Z 0x33 ; = 0x3c33


; :::
#subruledef opcode
{
    A => 0x1
    B => 0x2
    C => 0x3
}

#subruledef condition
{
    X => 0xa
    Y => 0xb
    Z => 0xc
}

#subruledef instruction
{
    {opc: opcode},                  => opc @ 0xa ; default to X
    {opc: opcode}-{cnd: condition}, => opc @ cnd
}

#ruledef
{
    {ins: instruction} {val: u8} => ins @ val
}

A,   0x33 ; = 0x1a33
B,   0x33 ; = 0x2a33
C,   0x33 ; = 0x3a33
A-X, 0x33 ; = 0x1a33
A-Y, 0x33 ; = 0x1b33
A-Z, 0x33 ; = 0x1c33
B-X, 0x33 ; = 0x2a33
B-Y, 0x33 ; = 0x2b33
B-Z, 0x33 ; = 0x2c33
C-X, 0x33 ; = 0x3a33
C-Y, 0x33 ; = 0x3b33
C-Z, 0x33 ; = 0x3c33


; :::
#subruledef big_endian
{
    PUSH {rlist: u8}     => 0b1101010 @ 0b0 @ rlist
    PUSH {rlist: u8}, LR => 0b1101010 @ 0b1 @ rlist
}

#ruledef little_endian
{
    {val: big_endian} => val[7:0] @ val[15:8]
}

PUSH 0b00001111     ; = 0x0fd4
PUSH 0b00001111, LR ; = 0x0fd5