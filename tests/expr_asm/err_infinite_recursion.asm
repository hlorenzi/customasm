#ruledef
{
    test {x} => asm { test {x} }
}

test 0x12 ; error: failed / note:_:3: within / error:_:3: failed / error:_:3: recursion