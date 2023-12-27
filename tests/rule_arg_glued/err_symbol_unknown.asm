#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

x = 0x12
ld ry ; error: failed / note:_:3: within / error: unknown