; ::: include
#ruledef
{
    test {value:i32} => value[31:16] @ 0x00 @ value
}

; :::
test 0            ; = 0x0000_00_00000000
; :::
test -0x7fff_ffff ; = 0x8000_00_80000001
; :::
test -0x0eee_eeee ; = 0xf111_00_f1111112
; :::
test -0x0000_0001 ; = 0xffff_00_ffffffff
; :::
test -0x0000_0002 ; = 0xffff_00_fffffffe
; :::
test -0x0001_0000 ; = 0xffff_00_ffff0000
; :::
test -0x0000_1000 ; = 0xffff_00_fffff000
; :::
test -0x1000_0000 ; = 0xf000_00_f0000000
; :::
test -0x1000_1000 ; = 0xefff_00_effff000
; :::
test -349678887   ; = 0xeb28_00_eb2852d9