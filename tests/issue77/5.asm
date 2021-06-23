#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x0000_0002 ; = 0xffff_00_fffffffe