#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld $pc
$pc: ; error: reserved
    ld $pc
    ld pc

; legacy: off