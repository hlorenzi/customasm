#ruledef
{
    test {x} => asm { emit {x} }
}

test 0x12 ; = 0x12

#ruledef
{
    emit {x: i8} => x
}