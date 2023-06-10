#ruledef test
{
    loop => 0x5555 @ $`16
}


ADDR_START = ADDR1 @ ADDR2
ADDR1 = (8 * X)`8

#bankdef test
{
    #addr ADDR_START
    #outp OUTPUT
}

OUTPUT = 8 * 0x0000
X = 0x10
ADDR2 = 0x00


loop ; = 0x55558000
loop ; = 0x55558004
loop ; = 0x55558008
loop ; = 0x5555800c
loop ; = 0x55558010
loop ; = 0x55558014
loop ; = 0x55558018
loop ; = 0x5555801c