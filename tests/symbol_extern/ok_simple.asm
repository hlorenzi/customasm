#ruledef test
{
    ld8  {x} => 0x55 @ x`8
    ld16 {x} => 0x66 @ x`16
}

#const(extern) global_var

ld8 global_var ; = 0x5500
; extern: global_var[8:0] @ 0x8
ld16 global_var ; = 0x660000
; extern: global_var[16:0] @ 0x18