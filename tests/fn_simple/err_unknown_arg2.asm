#ruledef test
{
    ld {x} => 0x55 @ x`8
}

#fn add1(value) => xxxx + 1

ld add1(0) ; error: failed / note:_:3: within / error: failed / note:_:6: within / error:_:6: unknown
ld add1(-2)
ld add1(2)
ld add1(0xffff)