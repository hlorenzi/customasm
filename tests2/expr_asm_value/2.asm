; concat
#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit x } @ asm { emit 0xff }
}

emit 0x12     ; = 0x12
test 0x12     ; = 0x12ff
test 0x10 + 2 ; = 0x12ff