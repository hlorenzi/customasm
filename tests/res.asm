; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

ld $ ; = 0x5500
#res 0
ld $ ; = 0x5502

; :::

ld $ ; = 0x5500
#res 1 ; = 0x00
ld $ ; = 0x5503

; :::

ld $ ; = 0x5500
#res 1 + 2 ; = 0x000000
ld $ ; = 0x5505

; :::

ld $ ; = 0x5500
#res 2 ; = 0x0000
#res 3 ; = 0x000000
ld $ ; = 0x5507

; :::

ld $ ; = 0x5500
#res 2 ; = 0x0000
label:
#res 3 ; = 0x000000
ld $ ; = 0x5507
ld label ; = 0x5504

; :::

#bankdef test { #bits 3, #addr 0, #outp 0 }

#d3 $ ; = 0b000
#res 2 ; = 0b000_000
label:
#res 3 ; = 0b000_000_000
#d3 $ ; = 0b110
#d3 label ; = 0b011

; :::

#res -1 ; error: valid range

; :::

#res 0x1_0000_0000_0000_0000 ; error: valid range