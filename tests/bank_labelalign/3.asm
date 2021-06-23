#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#labelalign 32
#bankdef a { #addr 0, #size 8, #outp 0 }
#labelalign 64
#bankdef b { #addr 0, #outp 8 * 8 }

#bank a
ld $
label1:
ld $

#bank b
ld $
label2:
ld $

; = 0x5500000055040000_55000000000000005508