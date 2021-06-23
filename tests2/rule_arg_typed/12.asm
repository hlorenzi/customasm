#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld -1 ; error: out of range