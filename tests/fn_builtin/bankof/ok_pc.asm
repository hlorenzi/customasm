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

; padding for bank offset
; = 0x0000

#d16 bankof($).addr ; = 0xabcd
#d16 bankof($).outp ; = 0x0010
#d16 bankof($).bits ; = 0x0008
#d16 bankof($).size ; = 0x00ef
#d16 bankof($).size_b ; = 0x0778
#d16 bankof($).data.a ; = 0x00aa
#d16 bankof($).data.b ; = 0x00bb