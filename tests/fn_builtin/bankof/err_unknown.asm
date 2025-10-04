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

label:
    #d16 bankof(label).unknown ; error: failed / error: unknown symbol `unknown`