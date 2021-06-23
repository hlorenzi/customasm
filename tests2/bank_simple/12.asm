#ruledef test
{
    loop => 0x5555 @ $`16
}

#bankdef a { #outp 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff } ; error: valid range