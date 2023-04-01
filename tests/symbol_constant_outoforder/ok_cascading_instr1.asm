#ruledef test
{
    ld {x} =>
    {
        assert(x <= 0x8)
        0x11 @ x`16
    }

    ld {x} =>
    {
        assert(x > 0x8)
        0x22 @ x`8
    }
}

x1 = y1 + y1
ld x1 ; = 0x2210
ld x1 ; = 0x2210
y1 = 8

ld x2 ; = 0x110002
ld x2 ; = 0x110002
x2 = y2 + y2
y2 = 1