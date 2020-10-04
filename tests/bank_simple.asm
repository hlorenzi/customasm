; ::: include

#ruledef test
{
    loop => 0x5555 @ $`16
}

; :::
#bankdef test { #addr 0x8000 } ; = 0x
; :::
#bankdef test { #addr 0x8000, #size 0x10 } ; = 0x
; :::
#bankdef test { #addr 0x8000 + 0x2000, #size 0x10 * 8 } ; = 0x

; :::

#bankdef test
{
    #addr 0x8000
    #outp 8 * 0x0000
}

loop
loop
loop
loop
 ; = 0x55558000
 ; = 0x55558004
 ; = 0x55558008
 ; = 0x5555800c

; :::

#bankdef a
{
    #addr 0xaa00
    #size 0x0010
    #outp 8 * 0x0000
}

#bankdef b
{
    #addr 0xbb00
    #size 0x0010
    #outp 8 * 0x0010
}

loop
loop
#bank a
loop
loop
#bank b
loop
loop
#bank a
loop
loop
; = 0x5555aa00
; = 0x5555aa04
; = 0x5555aa08
; = 0x5555aa0c
; = 0x5555bb00
; = 0x5555bb04
; = 0x5555bb08
; = 0x5555bb0c

; :::

#bankdef test
{
    #addr 0x8000
    #outp 8 * 0x0000
}

loop ; = 0x55558000
loop ; = 0x55558004
loop ; = 0x55558008
loop ; = 0x5555800c
loop ; = 0x55558010
loop ; = 0x55558014
loop ; = 0x55558018
loop ; = 0x5555801c

; :::

#bankdef test
{
    #addr 0x8000
    #size 0x0008
    #outp 0x0000
}

loop
loop
loop ; error: bank range
loop ; error: bank range

; :::

#bankdef test { #addr 0x8000 }
loop ; error: non-writable

; :::
#bankdef a { #unk 0x0000 } ; error: unknown
; :::
#bankdef a { #outp -0x8000 } ; error: valid range
; :::
#bankdef a { #size 0x10 #outp 8 * 0x10 } ; error: expected line break
; :::
#bankdef a { #outp 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff } ; error: valid range
; :::
#bankdef a { #addr 0x8000 }
#bank c ; error: unknown
; :::
#bankdef a { #addr 0x8000 }
#bankdef a { #addr 0x8000 } ; error: duplicate