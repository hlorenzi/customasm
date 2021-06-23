#ruledef test
{
    ld {x} => 0x55 @ x[7:0]
}

ld ; error: no match