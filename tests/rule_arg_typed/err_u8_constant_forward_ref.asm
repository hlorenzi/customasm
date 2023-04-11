#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld x ; error: failed / note:_:3: within / error: out of range
x = 0x100