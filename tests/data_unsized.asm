; :::
#d 0x00 ; = 0x00
; :::
#d 0x0000 ; = 0x0000
; :::
#d 0x12 ; = 0x12
; :::
#d 0x1234 ; = 0x1234
; :::
#d 10`16 ; = 0x000a


; :::
#d 0x12, 0x345, 0x6, 0x789a ; = 0x123456789a


; :::
x = 0x1234
#d x, 0x56, x ; = 0x1234561234
; :::
#d x, 0x56, x ; = 0x1234561234
#d x, 0x56, x ; = 0x1234561234
x = 0x1234


; :::
#d x ; = 0x55
label:
#d16 label`16 ; = 0x0001
x = 0x55


; :::
#d 10 ; error: infer size
; :::
x = 10
#d x ; error: infer size
; :::
#d x ; error: infer size
x = 10
; :::
#d label ; error: infer size
label: