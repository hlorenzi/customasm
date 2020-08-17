; ::: include

#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x55 @ x`8
    }
}

; :::
ld 0x5 ; = 0x5505
; :::
ld -0x20 ; = 0x55e0
; :::
ld 0x15 ; error: failed to resolve
; :::
ld var ; = 0x5505
var = 0x5
; :::
ld var ; error: failed to resolve
var = 0x15