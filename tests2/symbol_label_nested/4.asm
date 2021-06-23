#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; multiple sections and multiple levels

global1:
    ld global1 ; = 0x5500
.local1_1:
    ld .local1_1 ; = 0x5502
..local2_1:
    ld ..local2_1 ; = 0x5504
...local3_1:
    ld ...local3_1 ; = 0x5506
..local2_2:
    ld ..local2_2 ; = 0x5508
...local3_1:
    ld ...local3_1 ; = 0x550a
...local3_2:
    ld ...local3_2 ; = 0x550c
..local2_3:
    ld ..local2_3 ; = 0x550e
.local1_2:
    ld .local1_2 ; = 0x5510
global2:
    ld global2 ; = 0x5512
    
    ld global1 ; = 0x5500
    ld global1.local1_1 ; = 0x5502
    ld global1.local1_1.local2_1 ; = 0x5504
    ld global1.local1_1.local2_1.local3_1 ; = 0x5506
    ld global1.local1_1.local2_2 ; = 0x5508
    ld global1.local1_1.local2_2.local3_1 ; = 0x550a
    ld global1.local1_1.local2_2.local3_2 ; = 0x550c
    ld global1.local1_1.local2_3 ; = 0x550e
    ld global1.local1_2 ; = 0x5510