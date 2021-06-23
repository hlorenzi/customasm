#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $
#align 3
ld $ ; error: failed / error: aligned