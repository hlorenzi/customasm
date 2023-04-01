#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#labelalign 32 ; error: deprecated
ld $
label:
ld $
.sublabel:
ld $