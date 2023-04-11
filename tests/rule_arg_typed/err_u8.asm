#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 0x123 ; error: failed / note:_:3: within / error: out of range
ld 256 ; error: failed / note:_:3: within / error: out of range
ld -1 ; error: failed / note:_:3: within / error: out of range
ld -0x1 ; error: failed / note:_:3: within / error: out of range