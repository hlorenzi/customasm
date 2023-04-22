#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r
    test {r: reg} => asm { emit {r} }
}


test 0x12 ; error: no match