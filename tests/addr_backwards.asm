; :::
#d8 0
#addr 0 ; error: behind

; :::
#addr -1 ; error: valid range

; :::
#d8 0
#d8 1
#addr 0x2
; = 0x0001

; :::
#d8 0
#d8 1
#d8 2
#addr 0x2 ; error: behind

; :::
#d8 0
#d8 1
#d8 2
#d8 3
#addr 0x2 ; error: behind

; :::
#bankdef non_writable { #addr 0, #size 10 }
#res 1
#res 1
#res 1
#res 1
#addr 0x2

; :::
#bankdef non_writable { #addr 0, #size 10 }
#res 1
#res 1
#res 1
#res 1
#addr -1 ; error: valid range