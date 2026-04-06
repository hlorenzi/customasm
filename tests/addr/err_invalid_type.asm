#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#addr {} ; error: invalid type
ld $