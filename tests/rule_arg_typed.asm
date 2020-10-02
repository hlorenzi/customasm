; ::: include

#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

; :::
ld 0 ; = 0x5500
; :::
ld 12 ; = 0x550c
; :::
ld 6 * 2 ; = 0x550c
; :::
ld 0xc ; = 0x550c
; :::
ld 0xff ; = 0x55ff
; :::
ld 255 ; = 0x55ff
; :::
ld 0x001 ; = 0x5501
; :::
x = 0xff
ld x ; = 0x55ff
; :::
ld x ; = 0x55ff
x = 0xff
; :::
ld 0x123 ; error: out of range
; :::
ld 256 ; error: out of range
; :::
ld -1 ; error: out of range
; :::
ld -0x1 ; error: out of range
; :::
x = 0x100
ld x ; error: out of range
; :::
ld x ; error: out of range
x = 0x100


; ===========
; ::: include

#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

; :::
ld 0 ; = 0x5500
; :::
ld 12 ; = 0x550c
; :::
ld 6 * 2 ; = 0x550c
; :::
ld 0xc ; = 0x550c
; :::
ld 0x001 ; = 0x5501
; :::
ld 0x7f ; = 0x557f
; :::
ld 127 ; = 0x557f
; :::
ld -1 ; = 0x55ff
; :::
ld -0x1 ; = 0x55ff
; :::
ld -128 ; = 0x5580
; :::
ld !0x7f ; = 0x5580
; :::
ld 128 ; error: out of range
; :::
ld 0x80 ; error: out of range
; :::
ld -129 ; error: out of range
; :::
ld !0x80 ; error: out of range


; ===========
; ::: include

#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

; :::
ld 0 ; = 0x5500
; :::
ld 12 ; = 0x550c
; :::
ld 6 * 2 ; = 0x550c
; :::
ld 0xc ; = 0x550c
; :::
ld 0x001 ; = 0x5501
; :::
ld 0xff ; = 0x55ff
; :::
ld 255 ; = 0x55ff
; :::
ld -1 ; = 0x55ff
; :::
ld -0x1 ; = 0x55ff
; :::
ld -128 ; = 0x5580
; :::
ld !0x7f ; = 0x5580
; :::
ld 256 ; error: out of range
; :::
ld 0x100 ; error: out of range
; :::
ld -129 ; error: out of range
; :::
ld !0x80 ; error: out of range


; ===========
; ::: include

#ruledef test
{
    ld1 {x: i1} => 0x55 @ 0b000 @ x
}

; :::
ld1 0 ; = 0x550
; :::
ld1 1 ; = 0x551
; :::
ld1 2 ; error: out of range
; :::
ld1 -1 ; = 0x551
; :::
ld1 -2 ; error: out of range


; ===========
; ::: include

#ruledef test
{
    ld32 {x: i32} => 0x55 @ x
}

; :::
ld32 0 ; = 0x5500000000
; :::
ld32 0xabcde ; = 0x55000abcde
; :::
ld32 -1 ; = 0x55ffffffff
; :::
ld32 1 << 32 ; error: out of range