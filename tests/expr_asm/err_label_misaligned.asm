#ruledef
{
    ld {addr: u8} => 0xee @ addr
    test => asm {
        start:
        ld 0x11
        ld 0x22
        ld start
        ld 0x33
    }
}

#d 0xf
test ; error: failed / note:_:4: within / error:_:5: not aligned / note: 4 more bits