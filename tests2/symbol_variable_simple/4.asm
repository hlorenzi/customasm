#ruledef test
{
    ld {x} => 0x55 @ x`8
}


val2 = val1 + val1 ; error: unknown
val1 = 2 * 2
ld val1
ld val2