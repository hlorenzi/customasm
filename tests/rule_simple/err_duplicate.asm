#ruledef test
{
    halt => 0x55
    nop => 0xaa
}


#ruledef test ; error: duplicate / note:_:1: first
{
    halt => 0x55
}