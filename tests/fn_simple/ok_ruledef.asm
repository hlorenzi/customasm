#ruledef test
{
    ld {x} => 0x55 @ add1(x)`8
}

#fn add1(value) => value + 1

ld 0     ; = 0x5501
ld -2    ; = 0x55ff
ld 2     ; = 0x5503
ld 0x100 ; = 0x5501