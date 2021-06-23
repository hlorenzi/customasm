#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x7fff_ffff ; = 0x8000_00_80000001