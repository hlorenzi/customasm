#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld le
le: ; error: reserved
    ld le
    ld le(0x04)

; legacy: on