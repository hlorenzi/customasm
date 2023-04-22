#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit {x} + 0x10 }
}

test 2     ; = 0x12
test 1 + 1 ; = 0x12