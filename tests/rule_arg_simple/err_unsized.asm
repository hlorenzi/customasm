#ruledef test
{
    ld {x} => 0x55 @ x
}

ld 123 ; error: failed / note:_:3: within / error:_:3: concatenation