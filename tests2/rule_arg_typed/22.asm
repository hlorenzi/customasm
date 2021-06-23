#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld 127 ; = 0x557f