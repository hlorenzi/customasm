#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

x = 0x100
ld x ; error: failed / note:_:3: within / error: out of range