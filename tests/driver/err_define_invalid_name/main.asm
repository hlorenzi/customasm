#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val = 0
ld val

; command: main.asm -o out.bin -d123=123
; error: unused define `123`