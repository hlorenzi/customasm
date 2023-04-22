#fn recursive(value) => recursive(value)

#d8 recursive(0) ; error: failed / error: failed / note:_:1: within / error:_:1: failed / error:_:1: recursion depth
#d8 recursive(-2)
#d8 recursive(2)