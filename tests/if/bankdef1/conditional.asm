#if x
{
    #bankdef a
    {
        addr = 0x0
        outp = 0x0
    }
}
#else
{
    #bankdef b
    {
        addr = 0x04
        outp = 0x04 * 8
    }
}