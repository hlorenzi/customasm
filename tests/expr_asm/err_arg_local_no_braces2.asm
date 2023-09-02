#ruledef
{
    emit {x: i8} => x
    test {x} => {
        y = 0x10
        asm { emit {x} + y }
    }
}

test 2 ; error: failed / note:_:4: within / note:_:6: emit 2 + y / error:_:6: failed / note:_:3: within / error:_:6: unknown symbol `y`