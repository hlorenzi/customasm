; ::: data1.bin

hello

; :::
#d incbin("data1.bin") ; = 0x0a68656c6c6f0a0a
; :::
x = "data1.bin"
#d incbin(x) ; = 0x0a68656c6c6f0a0a
; :::
#d incbin("unk") ; error: not found
; :::
#d incbin("unk" @ 0xffff) ; error: not found
; :::
#d x
#d incbin(x) ; error: not found
x = "data1.bin"
; :::
#d incbin() ; error: wrong
; :::
#d incbin("data1.bin", "data2.bin") ; error: wrong