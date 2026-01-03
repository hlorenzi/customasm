#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld pc ; = 0x5500
$pc:
    ld pc ; = 0x5502
    ld $pc ; = 0x5502

; legacy: on