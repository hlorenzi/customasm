#ruledef test
{
    ld {size}, {x} => {
        0x55 @ x`size @ 0x44
    }
}

ld 8, 0x33 ; = 0x553344
ld 16, 0x33 ; = 0x55003344
ld 16 + 16, 0x33 ; = 0x550000003344
label:
ld label + 3, 0x33 ; = 0x55003344