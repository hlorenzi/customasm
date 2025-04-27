#ruledef
{
    emit {x: i8} => x
    test {x} => {
        asm { emit {x} + y }
    }
}

test 2 ; = 0x03
y: