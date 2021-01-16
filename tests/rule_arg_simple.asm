; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x[7:0]
}

; :::
ld 0 ; = 0x5500
; :::
ld 12 ; = 0x550c
; :::
ld 0xff ; = 0x55ff
; :::
ld 0x123 ; = 0x5523
; :::
ld ; error: no match
; :::
ld x ; error: unknown


; ===========
; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::
ld 0 ; = 0x5500
; :::
ld 12 ; = 0x550c
; :::
ld 0xff ; = 0x55ff
; :::
ld 0x123 ; = 0x5523
; :::
ld ; error: no match
; :::
ld x ; error: unknown


; ===========
; ::: include

#ruledef test
{
    ld {x}, {y} => 0x55 @ x`8 @ y`8
}

; :::
ld 0x11, 0x22 ; = 0x551122
; :::
ld 0x11, ; error: no match
; :::
ld 0x11, x ; error: unknown


; ===========
; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x
}

; :::
ld 0x11 ; = 0x5511
; :::
ld 0x1122 ; = 0x551122
; :::
ld 0x112233 ; = 0x55112233
; :::
ld 123 ; error: failed to resolve


; ===========
; ::: include

#ruledef test
{
    ld {x} => x
}

; :::
ld 0x11 ; = 0x11
; :::
ld 0x1122 ; = 0x1122
; :::
ld 0x112233 ; = 0x112233
; :::
ld 123 ; error: size of instruction


; ===========
; ::: include

#ruledef test
{
    ld {x} => y
}

; :::
ld 0x11 ; error: failed to resolve
; :::
ld 0x1122 ; error: failed to resolve
; :::
ld 0x112233 ; error: failed to resolve
; :::
ld 123 ; error: failed to resolve