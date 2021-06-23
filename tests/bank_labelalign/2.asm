#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#bankdef a { #addr 0, #outp 0, #labelalign 32 }
ld $ ; = 0x5500
label: ; = 0x0000
ld $ ; = 0x5504