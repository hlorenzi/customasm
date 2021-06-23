#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1:
.local1:
    ld ..local2 ; = 0x5502
..local2: