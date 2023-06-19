#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val = 0
ld val

; command: main.asm -o out.bin -dval=abc=123
; error: invalid define argument `val=abc=123`