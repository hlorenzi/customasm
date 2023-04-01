#ruledef test
{
    loop => 0x5555 @ $`16
}

#bankdef a { #addr 0x8000 }
#bankdef a { #addr 0x8000 } ; error: duplicate / note:_:6: first