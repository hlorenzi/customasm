#ruledef test
{
    ld {x} => {
        $pc = 0x55
        $pc @ x`8
    }
}


ld $ ; error: failed / note:_:3: within / error:_:4: reserved
; legacy: off