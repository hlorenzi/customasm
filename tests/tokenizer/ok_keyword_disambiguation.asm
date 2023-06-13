#ruledef test
{
    ld asm asmx {asmx} => 0x11 @ asmx
    ld true truex {truex} => 0x22 @ truex
    ld false falsex {falsex} => 0x33 @ falsex
}

ld asm asmx 0xff ; = 0x11ff
ld true truex 0xff ; = 0x22ff
ld false falsex 0xff ; = 0x33ff