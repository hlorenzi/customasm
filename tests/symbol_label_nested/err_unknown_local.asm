#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1:
.local1:
    ld .local3 ; error: failed / note:_:3: within / error: unknown