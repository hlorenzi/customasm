#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $
#align 3
ld $ ; error: failed / note:_:3: within / error: not aligned / note: 6 more bits