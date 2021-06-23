#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x1000_0000 ; = 0xf000_00_f0000000