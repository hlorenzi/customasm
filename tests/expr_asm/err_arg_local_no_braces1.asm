#ruledef
{
    emit {x: i8} => x
    test {x} => {
        asm { emit {x} + x }
    }
}

test 2 ; error: failed / note:_:4: within / error:_:5: failed / note:_:3: within / error:_:5: unknown