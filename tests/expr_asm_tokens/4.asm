#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm
    {
        emit {r}
        emit b
        emit {r}
    }
}

test a ; = 0xaabbaa