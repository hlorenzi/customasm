#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 0x123 ; error: out of range