#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; going back multiple levels

global1:
.local1_1:
..local2_1:
...local3_1:
    ld global1 ; = 0x5500
    ld .local1_1 ; = 0x5500
    ld ..local2_1 ; = 0x5500
    ld ...local3_1 ; = 0x5500
global2:
    ld global2 ; = 0x5508