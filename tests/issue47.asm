; :::
#ruledef
{
    ld {addr} => 0x00 @ addr`8
    ld x      => 0xff
}

ld 0x11 ; = 0x0011
ld x ; = 0xff


; :::
#ruledef
{
    ld x      => 0xff
    ld {addr} => 0x00 @ addr`8
}

ld 0x11 ; = 0x0011
ld x ; = 0xff


; :::
#ruledef
{
    ld {addr} => 0x00 @ addr`8
    ld x      => 0xff @ 0x22
}

ld 0x11 ; = 0x0011
ld x ; = 0xff22


; :::
#ruledef
{
    ld ({addr})	   => 0x00 @ addr`8
    ld ({addr}, x) => 0xff @ addr`8
}

ld (0x11) ; = 0x0011
ld (0x22, x) ; = 0xff22


; :::
#ruledef
{
    ld ({addr})	   => 0x00 @ addr`8
    ld ({addr}, x) => 0xff @ addr`8
}

ld (0x11)
ld (0x22), x) ; error: no match