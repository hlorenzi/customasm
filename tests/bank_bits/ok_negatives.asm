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
#d8 -1
#res 1
#d8 -1
#bank bits16
#d16 -1
#res 1
#d16 -1
#bank bits8
#res 1
#d8 -1
; = 0xff00ff00ff000000_ffff0000ffff