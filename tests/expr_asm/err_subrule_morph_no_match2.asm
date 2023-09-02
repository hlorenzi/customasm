#subruledef reg
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
    emit {r: reg}  => r
    test {r: reg2} => asm { emit {r} }
}

test a
test b
test c ; error: failed / note:_:17: within / note:_:17: emit c / error:_:17: no match