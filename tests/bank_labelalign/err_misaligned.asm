#ruledef test
{
    ld {x} => 0x55 @ x`8
}

#bankdef a
{
    #addr 0
    #outp 0
    #labelalign 25
}

ld $
label: ; error: not aligned / note: needs 7 more bits
ld $