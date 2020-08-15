; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

label:
    ld label ; = 0x5500

; :::

label1:
    ld label1 ; = 0x5500
label2:
    ld label1 ; = 0x5500
    ld label2 ; = 0x5502

; :::

label:
    ld label
label: ; error: duplicate
    ld label