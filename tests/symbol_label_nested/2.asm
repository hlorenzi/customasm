#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; multiple sections of one level

global1:
    ld global1 ; = 0x5500
.local1:
    ld .local1 ; = 0x5502
.local2:
    ld .local1 ; = 0x5502
    ld .local2 ; = 0x5504
global2:
    ld global1 ; = 0x5500
    ld global2 ; = 0x5508
.local1:
    ld .local1 ; = 0x550c
.local2:
    ld .local1 ; = 0x550c
    ld .local2 ; = 0x550e