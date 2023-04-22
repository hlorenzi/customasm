#ruledef
{
    emit {x: i8}, {y: i8} => x @ y
    test {x}, {y} => asm { emit {x, y} }
}

test 0x12, 0x34 ; error: failed / note:_:4: within / error:_:4: expected `}`