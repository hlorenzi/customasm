#ruledef test
{
    ld {x} => 0x55 @ x`8
}


label:
    ld label
label: ; error: duplicate / note:_:7: first
    ld label