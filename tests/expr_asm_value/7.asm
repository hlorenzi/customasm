; from different ruledef blocks
#ruledef
{
    test {x} => asm { emit x }
}

#ruledef
{
    emit {x: i8} => x
}

emit 0x12 ; = 0x12
test 0x12 ; = 0x12