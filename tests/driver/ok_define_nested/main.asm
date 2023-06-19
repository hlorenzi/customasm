#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

val1 = 0
.val1 = 0
..val1 = 0
..val2 = 0
val2 = 0
ld val1
ld val1.val1
ld val1.val1.val1
ld val1.val1.val2
ld val2

; command: main.asm -fhexstr -o out.txt -dval1=0x55 -dval1.val1.val1=0x61 --define val1.val1.val2=0x62
; output: out.txt