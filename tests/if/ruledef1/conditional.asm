#if x
{
    #ruledef test
    {
        halt => 0x11
    }
}
#else
{
    #ruledef test
    {
        halt => 0x22
    }
}