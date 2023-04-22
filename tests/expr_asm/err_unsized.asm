#ruledef
{
    emit {x} => x
    test {x} => asm { emit {x} }
}

test 12 ; error: failed / note:_:4: within / error:_:4: failed / error:_:3: definite size