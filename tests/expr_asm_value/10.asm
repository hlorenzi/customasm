; multiple matches
#ruledef
{
    emit {x: i8} => x
    emit {x: i8} => x
    test {x} => asm { emit x }
}

test 0x12 ; error: failed / error:_:6: multiple / note:_:4: candidate / note:_:5: candidate