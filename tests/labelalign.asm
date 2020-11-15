; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

#labelalign 32
ld $ ; = 0x5500
label: ; = 0x0000
ld $ ; = 0x5504

; :::

ld $ ; = 0x5500
label1:
ld $ ; = 0x5502
#labelalign 64
label2: ; = 0x00000000
ld $ ; = 0x5508

; :::

#labelalign 33
ld $
label: ; error: not aligned
ld $