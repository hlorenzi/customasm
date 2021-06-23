#ruledef
{
    ld ({addr})	   => 0x00 @ addr`8
    ld ({addr}, x) => 0xff @ addr`8
}

ld (0x11) ; = 0x0011
ld (0x22, x) ; = 0xff22