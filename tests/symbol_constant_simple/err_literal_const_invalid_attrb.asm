#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#const(invalid) val = 0xaa ; error: invalid attribute
ld val