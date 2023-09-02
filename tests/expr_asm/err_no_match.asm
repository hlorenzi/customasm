#ruledef
{
    test {x} => asm { unknown {x} }
}

test 0x12 ; error: failed / note:_:3: within / note:_:3: unknown 0x12 / error:_:3: no match