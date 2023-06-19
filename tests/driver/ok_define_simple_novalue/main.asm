#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val1 = 0
val2 = {}
ld val1 ? 1 : 0
ld val2 ? 1 : 0

; command: main.asm -fhexstr -o out.txt -dval1 --define val2
; output: out.txt