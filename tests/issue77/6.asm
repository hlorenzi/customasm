#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x0001_0000 ; = 0xffff_00_ffff0000