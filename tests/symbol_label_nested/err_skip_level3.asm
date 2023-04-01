#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1:
    ld ..local2
..local2: ; error: nesting level