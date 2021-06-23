#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef reg2
{
    a => 0xaa
    b => 0xbb
    c => 0xcc
}

#ruledef
{
    emit {r: reg}  => r`8
    test {r: reg2} => asm { emit {r} }
}

test c ; error: failed / error:_:17: no match