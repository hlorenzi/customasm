#subruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r
    test {r: reg} => asm { emit {r} }
}


test c ; error: no match