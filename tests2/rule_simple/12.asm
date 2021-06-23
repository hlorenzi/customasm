#ruledef test
{
    halt => 0x55
    nop => 0xaa
}


#ruledef test ; error: duplicate
{
    halt => 0x55
}