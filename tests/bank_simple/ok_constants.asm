#ruledef test
{
    loop => 0x5555 @ $`16
}


ADDR_START = 0x8000
OUTPUT = 8 * 0x0000

#bankdef test
{
    #addr ADDR_START
    #outp OUTPUT
}


loop ; = 0x55558000
loop ; = 0x55558004
loop ; = 0x55558008
loop ; = 0x5555800c
loop ; = 0x55558010
loop ; = 0x55558014
loop ; = 0x55558018
loop ; = 0x5555801c