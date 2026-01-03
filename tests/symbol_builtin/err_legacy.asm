#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld pc
pc:
    ld pc
    ld $pc ; error: failed / note:_:3: within / error: unknown symbol `$pc`

; legacy: on