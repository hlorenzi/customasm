#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld -129 ; error: out of range