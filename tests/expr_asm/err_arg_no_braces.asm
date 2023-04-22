#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit x }
}

test 0x12 ; error: failed / note:_:4: within / error:_:4: failed / note:_:3: within / error:_:4: unknown