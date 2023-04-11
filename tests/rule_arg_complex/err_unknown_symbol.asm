#ruledef test
{
    ld {x} => 0x55 @ x`8
}

ld (3 + x) * 5 ; error: failed / note:_:3: within / error: unknown