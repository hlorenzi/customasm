#subruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r
    test {r: reg} => asm { emit r }
}

test a ; error: failed / note:_:10: within / error:_:10: no match