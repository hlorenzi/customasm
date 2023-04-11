#ruledef test
{
    ld1 {x: i8} => 0xaa @ x
    ld2 {x: s8} => 0xbb @ x
    ld3 {x: u8} => 0xcc @ x
}

ld1 "abc" ; error: failed / note:_:3: within / error: out of range
ld2 "ã" ; error: failed / note:_:4: within / error: out of range
ld3 "ü" ; error: failed / note:_:5: within / error: out of range