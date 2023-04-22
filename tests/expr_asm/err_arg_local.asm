#ruledef
{
    emit {x: i8} => x
    test {x} => {
        y = 0x10
        asm { emit {x} + {y} }
    }
}

test 2 ; error: failed / note:_:4: within / error:_:6: unknown