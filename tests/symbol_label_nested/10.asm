#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1:
.local1:
global2:
    ld .local1 ; error: failed / error: unknown