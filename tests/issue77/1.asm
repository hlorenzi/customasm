#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test 0            ; = 0x0000_00_00000000