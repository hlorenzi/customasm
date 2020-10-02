; ::: data1.txt

0001_1000

; ::: data2.txt

101

; ::: data3.txt

0101
1010
    1 1 1 1
0101
1010

; ::: data4.txt

0b1101

; :::
#d incbinstr("data1.txt") ; = 0x18
; :::
#d incbinstr("data2.txt") ; = 0b101
; :::
#d incbinstr("data3.txt") ; = 0x5af5a
; :::
#d incbinstr("data4.txt") ; error: invalid character
; :::
#d incbinstr("unk") ; error: not found
; :::
#d incbinstr("unk" @ 0xffff) ; error: not found
; :::
#d x
#d incbinstr(x) ; error: not found
x = "data1.txt"
; :::
#d incbinstr() ; error: wrong
; :::
#d incbinstr("data1.txt", "data2.txt") ; error: wrong 