#subruledef inner
{
     {x}   => 0x11 @ x`8
    &{x}   => 0x22 @ x`8
    ({x}+) => 0x33 @ x`8
}

#ruledef test
{
    ld {x: inner} => 0x55 @ x`16
}

ld x ; error: failed / note:_:10: within / note:_:3: within / error: unknown