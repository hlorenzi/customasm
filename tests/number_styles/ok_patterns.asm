#ruledef test
{
    ld $reg => $11
    ld %reg => $22
}

ld $reg ; = 0x11
ld %reg ; = 0x22