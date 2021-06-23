#ruledef test
{
    ld {x} => 0x55 @ x[7:0]
}

ld 12 ; = 0x550c