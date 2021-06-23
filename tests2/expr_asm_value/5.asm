; multiple captured variables
#ruledef
{
    emit {x: i8}, {y: i8} => x @ y
    test {x}, {y} => asm { emit x, y }
}

test 0x12, 0x34         ; = 0x1234
test 0x10 + 2, 0x30 + 4 ; = 0x1234