; ::: data1.txt

0180

; ::: data2.txt

5

; ::: data3.txt

00_00
5a 5a
    f
5a 5a
00_00

; ::: data4.txt

0x5a5a

; :::
#d inchexstr("data1.txt") ; = 0x0180
; :::
#d inchexstr("data2.txt") ; = 0x5
; :::
#d inchexstr("data3.txt") ; = 0x00005a5af5a5a0000
; :::
#d inchexstr("data4.txt") ; error: invalid character
; :::
#d inchexstr("unk") ; error: not found
; :::
#d inchexstr("unk" @ 0xffff) ; error: not found
; :::
#d x
#d inchexstr(x) ; error: not found
x = "data1.txt"
; :::
#d inchexstr() ; error: wrong
; :::
#d inchexstr("data1.txt", "data2.txt") ; error: wrong 