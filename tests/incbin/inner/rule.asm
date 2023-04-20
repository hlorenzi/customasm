#ruledef uses_incbin
{
    hlt => incbin("data1.bin")
    ld {x} => x
    st {x} => incbin(x)
}