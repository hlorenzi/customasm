#bankdef bank_a
{
    outp = 0
    addr = 0xabcd
    size = 0x1
    data = struct {
        a = 0xaa00
        b = 0xbb00
    }
}

#bankdef bank_b
{
    outp = 8
    addr = 0xeffe
    size = 0x10
    data = struct {
        a = 0x00aa
        b = 0x00bb
    }
}

#bank bank_a
label_a:
    #d 0xaa ; = 0xaa

#bank bank_b
label_b:
    #d 0xbb ; = 0xbb

    #d16 bankof(label_a).addr ; = 0xabcd
    #d16 bankof(label_a).data.a ; = 0xaa00
    #d16 bankof(label_a).data.b ; = 0xbb00

    #d16 bankof(label_b).addr ; = 0xeffe
    #d16 bankof(label_b).data.a ; = 0x00aa
    #d16 bankof(label_b).data.b ; = 0x00bb