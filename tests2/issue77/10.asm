#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -349678887   ; = 0xeb28_00_eb2852d9