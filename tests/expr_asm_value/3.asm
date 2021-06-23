; multi-line
#ruledef
{
    emit {x: i8} => 0x11 @ x
    load {x: i8} => 0x22 @ x
    test {x} => asm
    {
        emit x
        emit 0xff
        load x
    }
}

test 0x12 ; = 0x1112_11ff_2212