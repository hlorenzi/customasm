#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit {x} * 0x10 }
}

test 3         ; = 0x30
test 1 + 2     ; = 0x21
test 1 + 2 + 3 ; = 0x33