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

test a ; = 0xaa
test b ; = 0xbb
emit a ; = 0xaa
emit b ; = 0xbb