#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 6 * 2 ; = 0x550c