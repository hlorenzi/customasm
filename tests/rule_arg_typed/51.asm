#ruledef test
{
    ld32 {x: i32} => 0x55 @ x
}

ld32 0 ; = 0x5500000000