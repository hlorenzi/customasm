; assert resolution
#ruledef
{
    emit {x: i8} =>
    {
        assert(x < 0x10)
        0x11 @ x
    }
    emit {x: i8} =>
    {
        assert(x >= 0x10)
        0x22 @ x
    }
    test {x} => asm { emit x }
}

emit 0x08 ; = 0x1108
emit 0x12 ; = 0x2212
test 0x08 ; = 0x1108
test 0x12 ; = 0x2212