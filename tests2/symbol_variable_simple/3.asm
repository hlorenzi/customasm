#ruledef test
{
    ld {x} => 0x55 @ x`8
}


val1 = 2 * 2
val2 = val1 + val1
ld val1 ; = 0x5504
ld val2 ; = 0x5508