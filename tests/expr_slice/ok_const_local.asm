#ruledef test
{
    ld {x} => {
        size = 4 + 4
        0x55 @ x`size @ 0x44
    }
}

ld 0x33 ; = 0x553344