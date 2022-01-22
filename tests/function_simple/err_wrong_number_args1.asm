#fn add(value1, value2, value3) =>
{
    value1 + value2 + value3
}

#d8 add(0)          ; error: failed / error: wrong number
#d8 add(0, 1)       ; error: failed / error: wrong number
#d8 add(0, 1, 2, 3) ; error: failed / error: wrong number