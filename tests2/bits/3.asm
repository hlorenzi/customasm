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