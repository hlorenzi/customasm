#ruledef test
{
    ld {x} => 0x55 @ x
}

ld 123 ; error: failed / error:_:3: concatenation