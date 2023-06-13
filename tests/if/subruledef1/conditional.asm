#if x
{
    #ruledef test
    {
        ld {x: subtest} => 0x11 @ x
    }
}
#else
{
    #ruledef test
    {
        ld {x: subtest} => 0x22 @ x
    }
}


#if y
{
    #subruledef subtest
    {
        {x: u8} => 0xaa @ x
    }
}
#else
{
    #subruledef subtest
    {
        {x: u8} => 0xbb @ x
    }
}