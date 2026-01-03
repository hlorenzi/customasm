#ruledef test
{
    ld {x} => {
        $incbin = 0x55
        $incbin @ x`8
    }
}


ld $ ; error: failed / note:_:3: within / error:_:4: reserved
; legacy: off