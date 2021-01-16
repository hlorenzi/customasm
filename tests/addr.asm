; ::: include

#ruledef test
{
    ld {x} => 0x55 @ x`8
}

; :::

ld $ ; = 0x5500
#addr 0x02
ld $ ; = 0x5502

; :::

ld $ ; = 0x5500
#addr 0x02 + 0x02 ; = 0x0000
ld $ ; = 0x5504

; :::

ld $ ; = 0x5500
#addr 0x03 ; = 0x00
#addr 0x04 ; = 0x00
ld $ ; = 0x5504

; :::

ld $ ; = 0x5500
#addr 0x04 ; = 0x0000
#addr 0x06 ; = 0x0000
ld $ ; = 0x5506

; :::

#bankdef test { #addr 0x1_0000_0000_0000_0000, #outp 0 }

#ruledef test2
{
    ld2 {x} => 0x55 @ x`128
}

ld2 $ ; = 0x55_0000_0000_0000_0001_0000_0000_0000_0000
#addr 0x1_0000_0000_0000_0020 ; = 0x00_0000_0000_0000_0000_0000_0000_0000
ld2 $ ; = 0x55_0000_0000_0000_0001_0000_0000_0000_0020

; :::

#bankdef test { #bits 3, #addr 0, #outp 0 }

#d3 $ ; = 0b000
#addr 0b010 ; = 0b000
label:
#addr 0b100 ; = 0b000_000
#d3 $ ; = 0b100
#d3 label ; = 0b010

; :::

#addr 0x1_0000_0000_0000_0000 ; error: valid range