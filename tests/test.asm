; ::: include

#subruledef op
{
    {x} => x[7:0]
    &{x} => 0x66 @ x[7:0]
    {x}$ => 0x77 @ x[7:0]
}

#ruledef cpu6502
{
    halt => 0x55
    nop => 0xaa
    ld.x => 0xef
    ld {x} => 0x11 @ x[7:0]
    store {x:op} => 0x22 @ x[15:0]

    test1 {x} + 1 => 0x33 @ x[7:0]
    test2 {x: op}$ => 0x44 @ x[7:0]
}

; :::

halt ; = 0x55

; :::

nop ; = 0xaa

; :::

unk ; error: no match


;ld.x
;ld 5
;store 7
;store &8
;
;test1 5 + 1
;test1 5 + 1 + 1
;test1 (5 + 1) + 1
;
;test2 5$
;
;err
;err 123 err
;
;#use err