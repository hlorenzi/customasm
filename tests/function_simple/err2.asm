#ruledef test
{
    ld {x} => 0x55 @ x`8
}

#fn add1(value) => xxxx + 1

ld add1(0)      ; error: failed / error: failed / error:_:6: unknown
ld add1(-2)     ; error: failed / error: failed / error:_:6: unknown
ld add1(2)      ; error: failed / error: failed / error:_:6: unknown
ld add1(0xffff) ; error: failed / error: failed / error:_:6: unknown