#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; one level of nesting

global1:
    ld global1 ; = 0x5500
.local1:
    ld .local1 ; = 0x5502
.local2:
    ld .local1 ; = 0x5502
    ld .local2 ; = 0x5504