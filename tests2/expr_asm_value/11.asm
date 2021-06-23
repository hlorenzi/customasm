; inner error
#ruledef
{
    emit {x} => x / 0
    test {x} => asm { emit x }
}

test 12 ; error: failed / error:_:4: zero