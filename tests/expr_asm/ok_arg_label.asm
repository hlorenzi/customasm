#ruledef
{
    emit {x: i8} => x
    test {x} => {
        asm { emit {x} + y }
    }
}

y:
test 2 ; = 0x02