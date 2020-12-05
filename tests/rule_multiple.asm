; :::
#ruledef test
{
    halt => 0x55
    halt => 0x66
}

halt ; error: multiple matches


; ::: include
#ruledef test
{
    ld {val} => 0x00 @ val`8
    ld x     => 0xff @ 0x00
}
; :::
ld 0x55 ; = 0x0055
; :::
ld x ; = 0xff00
; :::
x = 0x55
ld x ; error: multiple matches
; :::
ld x ; error: multiple matches
x = 0x55