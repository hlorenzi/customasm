#ruledef
{
    emit {x: i8} => x
    test {x} => {
        asm { emit {x} + y.z }
    }
}

test 2 ; = 0x23
y:
.z = y + 0x20