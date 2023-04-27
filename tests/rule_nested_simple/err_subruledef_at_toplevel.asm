#subruledef inner
{
    a => 0x11
}

#ruledef test
{
    ld {reg: inner} => 0x55 @ reg`8
}

ld a
a ; error: no match