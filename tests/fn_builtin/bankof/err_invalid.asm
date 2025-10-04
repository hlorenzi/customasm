#bankdef test
{
    outp = 0x10
    addr = 0xabcd
    size = 0xef
    data = struct {
        a = 0xaa
        b = 0xbb
    }
}

#d16 bankof(123).addr ; error: failed / error: must have an associated symbol