#d { ; error: failed / error:_:3: invalid
    data = struct {}
    data.size = 4 + 4
    0xabcd`(data.size)
}