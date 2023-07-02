#ruledef
{
    emit {x: i8} => x
    emit {x: i8} => x
    test {x} => asm { emit {x} }
}

test 0x12 ; error: failed / note:_:5: within / error:_:5: multiple / note:_:3: match / note:_:4: match