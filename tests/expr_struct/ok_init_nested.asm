#d {
    data = struct {
        value = struct {
            hi = 0xab,
            lo = 0xcd,
        },
        size = 8 + 8
    }
    (data.value.hi @ data.value.lo)`(data.size)
} ; = 0xabcd