#d "abc" ; = 0x61_62_63
#d utf8("abc") ; = 0x61_62_63
#d ascii("abc") ; = 0x61_62_63
#d utf16be("abc") ; = 0x0061_0062_0063
#d utf16le("abc") ; = 0x6100_6200_6300
#d utf32be("abc") ; = 0x00000061_00000062_00000063
#d utf32le("abc") ; = 0x61000000_62000000_63000000

#d "àÿĀ" ; = 0xc3a0_c3bf_c480
#d utf8("àÿĀ") ; = 0xc3a0_c3bf_c480
#d ascii("àÿĀ") ; = 0xe0_ff_00
#d utf16be("àÿĀ") ; = 0x00e0_00ff_0100
#d utf16le("àÿĀ") ; = 0xe000_ff00_0001
#d utf32be("àÿĀ") ; = 0x000000e0_000000ff_00000100
#d utf32le("àÿĀ") ; = 0xe0000000_ff000000_00010000

#d "😀" ; = 0xf09f9880
#d utf8("😀") ; = 0xf09f9880
#d ascii("😀") ; = 0x00
#d utf16be("😀") ; = 0xd83d_de00
#d utf16le("😀") ; = 0x3dd8_00de
#d utf32be("😀") ; = 0x0001f600
#d utf32le("😀") ; = 0x00f60100

#d utf8(utf16be(utf32le(ascii("abc")))) ; = 0x61_62_63