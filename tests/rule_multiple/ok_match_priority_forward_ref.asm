#ruledef test
{
    ld {val} => 0x00 @ val`8
    ld x     => 0xff @ 0x00
}

ld x ; = 0xff00
x = 0x55