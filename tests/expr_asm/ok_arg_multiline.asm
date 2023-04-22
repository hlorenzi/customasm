#ruledef
{
    emit {x: i8} => 0x11 @ x
    test {x} => asm {
        emit {x}
        emit 0xff
        emit {x}
    }
}

test 0x12     ; = 0x1112_11ff_1112
test 0x10 + 2 ; = 0x1112_11ff_1112