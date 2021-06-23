#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x1000_1000 ; = 0xefff_00_effff000