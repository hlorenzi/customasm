; multiple matches
#ruledef
{
    emit {x: i8} => x
    emit {x: i8} => x
    test {x} => asm { emit x }
}

test 0x12 ; error: failed / error:_:6: multiple