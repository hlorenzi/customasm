#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
var1: #res 2 ; = 0x0000
var2: #res 3 ; = 0x000000
ld $ ; = 0x5507
ld var1 ; = 0x5502
ld var2 ; = 0x5504