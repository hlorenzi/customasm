#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1 ; error: no match
    ld global1