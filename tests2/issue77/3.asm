#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

test -0x0eee_eeee ; = 0xf111_00_f1111112