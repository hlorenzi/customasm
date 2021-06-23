#ruledef test
{
    loop => 0x5555 @ $`16
}


#bankdef test
{
    #addr 0x8000
    #outp 8 * 0x0000
}

loop
loop
loop
loop
 ; = 0x55558000
 ; = 0x55558004
 ; = 0x55558008
 ; = 0x5555800c