#ruledef
{
    move to {x: u8} if {y: u8} => 0x55 @ x @ y
}

input:
.input:
move to input if 0xaa ; = 0x5500aa
move to .input if 0xaa ; = 0x5500aa