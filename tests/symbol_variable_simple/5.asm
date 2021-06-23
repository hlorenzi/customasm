#ruledef test
{
    ld {x} => 0x55 @ x`8
}


val1 = 2 * 2
val2 = val1 + val1
ld val1
ld val2
val1 = 8 ; error: duplicate / note:_:7: first