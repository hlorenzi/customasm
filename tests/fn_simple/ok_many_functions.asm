#fn add1(value) => value + 1
#fn add2(value) => add1(add1(value))

#d8 add2(0)  ; = 0x02
#d8 add2(-3) ; = 0xff
#d8 add2(2)  ; = 0x04