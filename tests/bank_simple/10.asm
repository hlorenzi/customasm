#ruledef test
{
    loop => 0x5555 @ $`16
}

#bankdef a { #outp -0x8000 } ; error: valid range