#ruledef test
{
    loop => 0x5555 @ $`16
}

#bankdef test { #addr 0x8000, #size 0x10 } ; = 0x