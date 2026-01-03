#ruledef test
{
    ld {x} => 0x55 @ x`8
}


    ld $
$xxxx: ; error: reserved
    ld $

; legacy: off