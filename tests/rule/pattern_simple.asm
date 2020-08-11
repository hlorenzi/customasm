; ::: include

#ruledef simple
{
    halt => 0x55
}

; :::

halt ; = 0x55

; :::

h a l t ; = 0x55

; :::

HaLt ; = 0x55

; :::

halt ; = 0x55
halt ; = 0x55

; :::

halt
halt
; = 0x5555

; :::

unk ; error: no match

; :::

halt
unk ; error: no match
halt



; ===========
; ::: include

#ruledef simple
{
    halt => 0x55
    nop => 0xaa
}

; :::

halt ; = 0x55
nop  ; = 0xaa
halt ; = 0x55
nop  ; = 0xaa

; :::

halt
nop
halt nop ; error: no match
halt
nop



; ===========
; ::: include

#ruledef simple
{
    test*(x->$) => 0x55
}

; :::

test*(x->$) ; = 0x55

; :::

t e s t * ( x - > $ ) ; = 0x55