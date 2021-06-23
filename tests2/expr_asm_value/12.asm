; inner error
#ruledef
{
    emit {x} => x
    test {x} => asm { emit x }
}

test 12 ; error: failed / error:_:5: infer