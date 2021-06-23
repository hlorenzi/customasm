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