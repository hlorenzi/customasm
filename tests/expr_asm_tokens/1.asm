#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm { emit {r} }
}


emit a ; = 0xaa
emit b ; = 0xbb
test a ; = 0xaa
test b ; = 0xbb