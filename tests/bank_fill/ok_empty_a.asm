#bankdef a
{
    #bits 8
    #addr 0x0
    #size 0x4
    #outp 0x0
    #fill
}

#bankdef b
{
    #bits 8
    #addr 0x0
    #size 0x4
    #outp 8 * 0x4
    #fill
}


#bank b
#d8 4, 3, $
; = 0x00000000_04030200