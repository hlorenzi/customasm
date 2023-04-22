#ruledef
{
    emit {x: i8} => x
    test {x} => {
        y = x
        asm { emit {y} * 0x10 }
    }
}

test 3         ; = 0x30
test 1 + 2     ; = 0x30
test 1 + 2 + 3 ; = 0x60