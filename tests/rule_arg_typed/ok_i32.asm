#ruledef test
{
    ld32 {x: i32} => 0x55 @ x
}

ld32 0 ; = 0x5500000000
ld32 0xabcde ; = 0x55000abcde
ld32 -1 ; = 0x55ffffffff