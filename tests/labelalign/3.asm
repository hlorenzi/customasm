#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#labelalign 33
ld $
label: ; error: not aligned
ld $