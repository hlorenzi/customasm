#d {
    data = struct {
        value = 0xab @ 0xcd
        size = 8 + 8
    }
    data.value`(data.size)
} ; = 0xabcd