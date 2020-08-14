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
ld x ; error: failed to resolve


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
ld x ; error: failed to resolve


; ===========
; ::: include
; :::

#ruledef test
{
    ld {x} => 0x55 @ x ; error: size of rule production
}


