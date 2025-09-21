#const data = struct { value = 0xfe }

#if data.unknown == 0xfe ; error: unknown symbol `unknown`
{
    #d8 0xaa
}
#else
{
    #d8 0xbb
}