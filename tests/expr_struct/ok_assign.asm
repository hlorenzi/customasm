#d {
    data = struct { size = 4 + 4 }
    other = data
    0xabcd`(other.size)
} ; = 0xcd