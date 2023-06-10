#ruledef test
{
    loop => 0x5555 @ $`16
}

OUTPUT = 8 * 0x0000

#bankdef test
{
    #addr $ ; error: cannot get address
    #outp OUTPUT
}

loop