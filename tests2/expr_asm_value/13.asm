; from forward-referenced ruledef
#ruledef
{
    test {x} => asm { emit x }
}

; weird error?!
test 0x12 ; error: converge

#ruledef
{
    emit {x: i8} => x
}