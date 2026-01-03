#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld le ; = 0x5502
le:
    ld le ; = 0x5502
    ld $le(0x04) ; = 0x5504

; legacy: off