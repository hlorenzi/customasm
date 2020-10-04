; :::

#bits 3

#ruledef
{
    ld {x: u3} => 0b111 @ x
}

ld 0 ; = 0b111_000
label:
ld 5 ; = 0b111_101
ld 7 ; = 0b111_111
ld $ ; = 0b111_110
ld label ; = 0b111_010


; :::

#bits 3

label1:
#d 0b111
label2:
#d 0b101011
label3:
#d 0b1
label4: ; error: aligned


; :::

#bits 24

#ruledef
{
    ld {x: u16} => 0x55 @ x
}

ld 0 ; = 0x550000
label:
ld 0x1234 ; = 0x551234
ld label ; = 0x550001
ld $ ; = 0x550003