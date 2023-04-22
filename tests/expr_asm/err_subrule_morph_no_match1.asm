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
test c
emit a
emit b
emit c ; error: no match