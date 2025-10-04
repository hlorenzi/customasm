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

label:
    #d16 bankof(label).addr ; = 0xabcd
    #d16 bankof(label).outp ; = 0x0010
    #d16 bankof(label).bits ; = 0x0008
    #d16 bankof(label).size ; = 0x00ef
    #d16 bankof(label).size_b ; = 0x0778
    #d16 bankof(label).data.a ; = 0x00aa
    #d16 bankof(label).data.b ; = 0x00bb