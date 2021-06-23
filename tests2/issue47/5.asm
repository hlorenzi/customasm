#ruledef
{
    ld ({addr})	   => 0x00 @ addr`8
    ld ({addr}, x) => 0xff @ addr`8
}

ld (0x11)
ld (0x22), x) ; error: no match