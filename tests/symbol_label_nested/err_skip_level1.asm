#ruledef test
{
    ld {x} => 0x55 @ x`8
}


.local1: ; error: nesting level
    ld .local1