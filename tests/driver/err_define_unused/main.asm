#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val = 0
ld val

; command: main.asm -o out.bin -dvalue=0x55
; error: unused define `value`