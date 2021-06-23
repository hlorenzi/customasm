#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld -128 ; = 0x5580