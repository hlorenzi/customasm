#fn add1(value) => xxxx + 1

#d8 add1(0)      ; error: failed / error:_:1: unknown
#d8 add1(-2)     ; error: failed / error:_:1: unknown
#d8 add1(2)      ; error: failed / error:_:1: unknown
#d8 add1(0xffff) ; error: failed / error:_:1: unknown