#const data = struct { value = 0xfe }

#if data.value == 0xfe
{
    #d8 0xaa ; = 0xaa
}
#else
{
    #d8 0xbb
}