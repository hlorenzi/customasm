; ::: include

#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

; :::
ld r0 ; = 0x5500
; :::
ld r 0 ; = 0x5500
; :::
ld r12 ; = 0x550c
; :::
ld r257 ; = 0x5501
; :::
ld r 0xff ; = 0x55ff
; :::
ld r0xff ; error: no match
; :::
ld r0x123 ; error: no match
; :::
ld 0 ; error: no match
; :::
x = 0
ld rx ; error: no match


; ===========
; ::: include

#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

; :::
ld r0, 0x12 ; = 0x550012
; :::
ld r 0 , 0x34 ; = 0x550034
; :::
ld r257, 0x102 ; = 0x550102
; :::
ld r 0xff, 0x12 ; = 0x55ff12
; :::
ld r0xff, 0x12 ; error: no match
; :::
ld r0x123, 0x12 ; error: no match
; :::
ld 0, 0x12 ; error: no match
; :::
x = 0
ld rx, x ; error: no match