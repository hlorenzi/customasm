#ruledef
{
    test {x} => asm { test {x} }
}

test 0x12 ; error: failed / note:_:3: within / error:_:3: failed / note:_:3: test 0x12 / error:_:3: recursion