#ruledef test
{
    ld {x} => 0x55 @ x`8
}


global1:
..local1: ; error: nesting level
    ld ..local1