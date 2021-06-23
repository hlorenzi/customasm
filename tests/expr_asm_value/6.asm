; captured local variable
#ruledef
{
    emit {x: i8} => x
    test {x} =>
    {
        y = 0x10
        asm { emit x + y }
    }
}

test 2 ; = 0x12