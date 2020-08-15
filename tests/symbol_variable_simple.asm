; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

val = 0xaa
ld val ; = 0x55aa

; :::

val = 1 + 1
ld val ; = 0x5502

; :::

val1 = 2 * 2
val2 = val1 + val1
ld val1 ; = 0x5504
ld val2 ; = 0x5508

; :::

val1 = 2 * 2
val2 = val1 + val1
ld val1
ld val2
val1 = 8 ; error: duplicate