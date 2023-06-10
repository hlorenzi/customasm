#ruledef test
{
    loop => 0x5555 @ $`16
}

ADDR_START = 0x8000 | ADDR2
ADDR2 = $
OUTPUT = 8 * 0x0000

#bankdef test
{
    #addr ADDR_START ; error: unresolved
    #outp OUTPUT
}

loop