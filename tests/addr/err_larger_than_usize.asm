#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#addr 0x1_0000_0000_0000_0000 ; error: supported range
ld $