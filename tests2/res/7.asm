#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#res -1 ; error: valid range