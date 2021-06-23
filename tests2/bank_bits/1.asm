#bankdef bits8
{
    #bits 8
    #addr 0x0
    #size 0x8
    #outp 0x0
}

#bankdef bits16
{
    #bits 16
    #addr 0x0
    #size 0x8
    #outp 8 * 0x8
}


#bank bits8
#d8 0, 1, 2, 3, $
#bank bits16
#d16 0, 1, $
; = 0x0001020304000000_000000010002