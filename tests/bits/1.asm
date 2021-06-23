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