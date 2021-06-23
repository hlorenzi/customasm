#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 0x100 ; error: failed / error: out of range