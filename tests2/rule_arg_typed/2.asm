#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 12 ; = 0x550c