#ruledef test
{
    ld1 {x: i8} => 0xaa @ x
    ld2 {x: s8} => 0xbb @ x
    ld3 {x: u16} => 0xcc @ x
}

ld1 "a" ; = 0xaa61
ld2 "b" ; = 0xbb62
ld3 "c" ; = 0xcc0063