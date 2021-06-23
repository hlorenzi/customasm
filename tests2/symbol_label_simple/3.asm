#ruledef test
{
    ld {x} => 0x55 @ x`8
}


label:
    ld label
label: ; error: duplicate
    ld label