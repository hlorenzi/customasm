#ruledef test
{
    ld {x}, {y} => 0x55 @ x`8 @ y`8
}

ld 0x11, x ; error: failed / note:_:3: within / error: unknown