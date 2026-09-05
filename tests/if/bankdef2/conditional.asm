#if x
{
    #bankdef main
    {
        addr = 0x0
        outp = 0x0
    }
}
#else
{
    #bankdef main
    {
        addr = 0x04
        outp = 0x04 * 8
    }
}