#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val = 0
ld val

; command: main.asm -o out.bin -dval=abc
; error: invalid value for define `val`