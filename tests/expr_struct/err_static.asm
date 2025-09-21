#const data = struct { addr = 0xfe }

#bankdef bank {
    outp = 0
    addr = data.unknown ; error: unknown symbol `unknown`
    size = 8
}

#d8 $