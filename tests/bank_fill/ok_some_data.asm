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


#bank a
#d8 0, 1, $
#bank b
#d8 4, 3, $
; = 0x00010200_04030200