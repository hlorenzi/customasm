#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 256 ; error: failed / note:_:3: within / error: out of range
ld 0x100 ; error: failed / note:_:3: within / error: out of range
ld -129 ; error: failed / note:_:3: within / error: out of range
ld !0x80 ; error: failed / note:_:3: within / error: out of range