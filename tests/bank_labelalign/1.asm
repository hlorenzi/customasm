#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#labelalign 32
#bankdef a { #addr 0, #outp 0 }
ld $ ; = 0x5500
label: ; = 0x0000
ld $ ; = 0x5504