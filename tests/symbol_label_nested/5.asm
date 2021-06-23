#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; referencing symbols at levels above

global1:
.local1_1:
..local2_1:
...local3_1:
    ld global1 ; = 0x5500
    ld .local1_1 ; = 0x5500
    ld ..local2_1 ; = 0x5500
    ld ...local3_1 ; = 0x5500
..local2_2:
...local3_1:
    ld global1 ; = 0x5500
    ld .local1_1 ; = 0x5500
    ld ..local2_1 ; = 0x5500
    ld ...local3_1 ; = 0x5508
    ld ..local2_2 ; = 0x5508
    ld ...local3_1 ; = 0x5508
    ld ..local2_1.local3_1 ; = 0x5500