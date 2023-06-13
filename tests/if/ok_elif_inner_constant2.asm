#d 0xab ; = 0xab

x = y

#if false
{
    y = 0x11
}
#elif false
{
    y = 0x22
}
#elif true
{
    y = 0x33
}
#elif true
{
    y = 0x44
}
#else
{
    y = 0x55
}

#d y ; = 0x33