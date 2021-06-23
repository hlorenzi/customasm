#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld x ; error: failed / error: out of range
x = 0x100