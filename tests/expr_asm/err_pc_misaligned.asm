#ruledef
{
    ld {addr: u8} => 0xee @ addr
    test => asm {
        ld 0x11
        ld 0x22
        ld $
        ld 0x33
    }
}

#d 0xf
test ; error: failed / note:_:4: within / error:_:7: failed / note:_:3: within / error:_:7: not aligned / note: 4 more bits