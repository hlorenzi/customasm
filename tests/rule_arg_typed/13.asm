#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld -0x1 ; error: failed / error: out of range