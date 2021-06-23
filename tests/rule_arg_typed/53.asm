#ruledef test
{
    ld32 {x: i32} => 0x55 @ x
}

ld32 -1 ; = 0x55ffffffff