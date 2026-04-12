#d "abc"
#d $utf8("abc")
#d $ascii("abc")
#d $utf16be("abc")
#d $utf16le("abc")
#d $utf32be("abc")
#d $utf32le("abc")

#d "àÿĀ"
#d $utf8("àÿĀ")
#d $ascii("àÿĀ")
#d $utf16be("àÿĀ")
#d $utf16le("àÿĀ")
#d $utf32be("àÿĀ")
#d $utf32le("àÿĀ")

#d "😀"
#d $utf8("😀")
#d $ascii("😀")
#d $utf16be("😀")
#d $utf16le("😀"x) ; error: expected
#d $utf32be("😀")
#d $utf32le("😀")
