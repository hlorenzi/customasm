#ruledef test
{
    ld {x} => 0x55 @ x`8
}

ld x ; error: failed / note:_:3: within / error: unknown