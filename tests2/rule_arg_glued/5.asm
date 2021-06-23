#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

ld r 0xff ; = 0x55ff