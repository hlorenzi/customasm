#ruledef test
{
    ld {x} => 0x55 @ x[7:0]
}

ld 0x123 ; = 0x5523