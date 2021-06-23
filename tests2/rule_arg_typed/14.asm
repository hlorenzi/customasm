#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

x = 0x100
ld x ; error: out of range