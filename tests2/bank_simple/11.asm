#ruledef test
{
    loop => 0x5555 @ $`16
}

#bankdef a { #size 0x10 #outp 8 * 0x10 } ; error: expected line break