; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

ld $ ; = 0x5500
#align 8
ld $ ; = 0x5502

; :::

ld $ ; = 0x5500
#align 16 + 16 ; = 0x0000
ld $ ; = 0x5504

; :::

ld $ ; = 0x5500
#align 24 ; = 0x00
#align 32 ; = 0x00
ld $ ; = 0x5504

; :::

ld $ ; = 0x5500
#align 32 ; = 0x0000
#align 24 ; = 0x0000
ld $ ; = 0x5506

; :::

#bankdef test { #bits 3, #addr 0, #outp 0 }

#d3 $ ; = 0b000
#align 6 ; = 0b000
label:
#align 12 ; = 0b000_000
#d3 $ ; = 0b100
#d3 label ; = 0b010

; :::

ld $
#align 3
ld $ ; error: aligned

; :::

#align 0 ; error: valid range

; :::

#align -1 ; error: valid range

; :::

#align 0x1_0000_0000_0000_0000 ; error: valid range