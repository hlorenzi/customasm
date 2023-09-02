#ruledef
{
    emit {x: i8} => {
        assert(x < 0x10)
        x
    }
    emit {x: i8} => {
        assert(x >= 0x10 && x < 0x20)
        x
    }
    test {x} => asm { emit {x} }
}

test 0x30 ; error: failed / note:_:11: within / error:_:11: failed / note:_:11: emit 0x30 / note:_:3: within / error:_:4: assertion / note:_:7: within / error:_:8: assertion