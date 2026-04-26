#ruledef test
{
    ld {x} => 0x55 @ x`16
}

#const(extern) global_var

ld global_var`8 ; = 0x550000
; extern: global_var[8:0] @ 0x10