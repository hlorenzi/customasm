; ::: include

#ruledef test
{
    ld {x} =>
    {
        assert(x <= 0x8)
        0x11 @ x`16
    }

    ld {x} =>
    {
        assert(x > 0x8)
        0x22 @ x`8
    }
}

; :::
    ld label ; = 0x110006
    ld label ; = 0x110006
label:
; :::
    ld label ; error: converge
    ld label ; error: converge
    ld label ; error: converge
label:
; :::
    ld label ; error: converge
    ld 0x4
    ld label ; error: converge
label: