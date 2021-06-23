#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; multiple sections of one level, externally referenced

global1:
    ld global1 ; = 0x5500
.local1:
    ld .local1 ; = 0x5502
.local2:
    ld .local2 ; = 0x5504
global2:
    ld global1 ; = 0x5500
    ld global2 ; = 0x5506
.local1:
    ld global1.local1 ; = 0x5502
    ld global2.local1 ; = 0x550a
    ld .local1 ; = 0x550a
.local2:
    ld global1.local1 ; = 0x5502
    ld global1.local2 ; = 0x5504
    ld global2.local1 ; = 0x550a
    ld global2.local2 ; = 0x5510
    ld .local1 ; = 0x550a
    ld .local2 ; = 0x5510