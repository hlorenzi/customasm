#ruledef
{
    emit {x: i8} => x
    test {x} => {
        asm { emit {x} + y.z }
    }
}

test 2 ; = 0x22
y:
.z = 0x20