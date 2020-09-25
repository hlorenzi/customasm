; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::
; one level of nesting

global1:
    ld global1 ; = 0x5500
.local1:
    ld .local1 ; = 0x5502
.local2:
    ld .local1 ; = 0x5502
    ld .local2 ; = 0x5504

; :::
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

; :::
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

; :::
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

; :::
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

; :::
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

; :::

global1:
.local1:
    ld .local3 ; error: unknown

; :::

global1:
.local1:
    ld global1.local3 ; error: unknown

; :::

global1:
.local1:
    ld global2.local1 ; error: unknown

; :::

global1:
.local1:
global2:
    ld .local1 ; error: unknown

; :::

global1 ; error: no match
    ld global1

; :::

.local1: ; error: nesting level
    ld .local1

; :::

global1:
..local1: ; error: nesting level
    ld ..local1

; :::

.local1: ; error: nesting level
    ld .local1

; :::

global1:
.local1:
..local2 ; error: expected
    ld global1.local1
