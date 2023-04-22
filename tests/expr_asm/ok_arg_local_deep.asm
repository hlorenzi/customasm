#ruledef
{
    emit {x: i8} => x
    
    test2 {x} => {
        y = 0x10 + x
        asm { emit {y} }
    }

    test1 {x} => {
        y = 0x10 + x
        asm { test2 {y} }
    }
}

test1 2 ; = 0x22