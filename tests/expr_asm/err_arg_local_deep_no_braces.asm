#ruledef
{
    emit {x: i8} => x
    
    test2 {x} => {
        y = 0x10 + x
        asm { emit y }
    }

    test1 {x} => {
        y = 0x10 + x
        asm { test2 {y} }
    }
}

test1 2 ; error: failed / note:_:10: within / note:_:12: test2 :y / error:_:12: failed / note:_:5: within / error:_:7: failed / note:_:3: within / error:_:7: unknown