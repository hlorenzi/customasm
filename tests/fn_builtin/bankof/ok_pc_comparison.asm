#bankdef bank_a
{
    outp = 0
    addr = 0xabcd
    size = 5
}

#bankdef bank_b
{
    outp = 8 * 5
    addr = 0xeffe
    size = 5
}

#bank bank_a
label_a:
    #d bankof(label_a) == bankof($)       ? 0xff : 0x00 ; = 0xff
    #d bankof(label_b) == bankof($)       ? 0xff : 0x00 ; = 0x00
    #d bankof(label_a) == bankof(label_a) ? 0xff : 0x00 ; = 0xff
    #d bankof(label_a) == bankof(label_b) ? 0xff : 0x00 ; = 0x00
    #d bankof($)       == bankof($)       ? 0xff : 0x00 ; = 0xff

#bank bank_b
label_b:
    #d bankof(label_a) == bankof($)       ? 0xff : 0x00 ; = 0x00
    #d bankof(label_b) == bankof($)       ? 0xff : 0x00 ; = 0xff
    #d bankof(label_b) == bankof(label_b) ? 0xff : 0x00 ; = 0xff
    #d bankof(label_a) == bankof(label_b) ? 0xff : 0x00 ; = 0x00
    #d bankof($)       == bankof($)       ? 0xff : 0x00 ; = 0xff