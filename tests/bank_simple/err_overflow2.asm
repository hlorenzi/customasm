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
loop ; error: out of range for bank / note:_:7: defined here