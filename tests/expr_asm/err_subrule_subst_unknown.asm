#subruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r
    test {r: reg} => asm { emit {x} }
}

test a ; error: failed / note:_:10: within / error:_:10: unknown