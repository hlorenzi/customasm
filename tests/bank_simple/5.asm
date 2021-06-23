#ruledef test
{
    loop => 0x5555 @ $`16
}


#bankdef a
{
    #addr 0xaa00
    #size 0x0010
    #outp 8 * 0x0000
}

#bankdef b
{
    #addr 0xbb00
    #size 0x0010
    #outp 8 * 0x0010
}

loop
loop
#bank a
loop
loop
#bank b
loop
loop
#bank a
loop
loop
; = 0x5555aa00
; = 0x5555aa04
; = 0x5555aa08
; = 0x5555aa0c
; = 0x5555bb00
; = 0x5555bb04
; = 0x5555bb08
; = 0x5555bb0c