#ruledef test
{
    ld32 {x: i32} => 0x55 @ x
}

ld32 0xabcde ; = 0x55000abcde