#ruledef test
{
    loop => 0x5555 @ $`16
}


#bankdef test
{
    #addr 0x8000
    #size 0x0008
    #outp 0x0000
}

loop
loop
loop ; error: bank range
loop ; error: bank range