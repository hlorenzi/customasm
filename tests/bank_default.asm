; :::

#d8 1, 2, 3, 4 ; = 0x01020304

; :::

#d8 1, 2, 3, 4
#bankdef a_new_bank {} ; error: default bank

; :::

#res 4
#bankdef a_new_bank {} ; error: default bank

; :::

x = 0x25
label:
#bankdef a_new_bank {}
; = 0x