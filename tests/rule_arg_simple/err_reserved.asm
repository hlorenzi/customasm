#ruledef test
{
    ld {$le} => 0x55 @ $le ; error: reserved
}

ld 0x11
; legacy: off