#subruledef inner
{
    a => 0x11
}

#ruledef test
{
    ld {reg: unk} => 0x55 @ reg`8 ; error: unknown
}