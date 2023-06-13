#ruledef test
{
    ld {x: subtest} => 0x11 @ x
}


#if x
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


x = true
ld 0x55 ; = 0x11aa55