#ruledef
{
    ld {addr} => 0x00 @ addr`8
    ld x      => 0xff
}

ld 0x11 ; = 0x0011
ld x ; = 0xff