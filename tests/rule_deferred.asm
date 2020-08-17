; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::
ld label ; = 0x5502
label:
; :::
ld var ; = 0x5511
var = 0x11