#const data = struct { addr = 0xfe }

#bankdef bank {
    outp = 0
    addr = data.addr
    size = 8
}

#d8 $ ; = 0xfe