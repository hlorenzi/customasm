#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

val1 = 0
val2 = {}
val3 = false
val4 = 1
ld val1
ld val2
ld val3
ld val4

; command: main.asm -fhexstr -o out.txt -dval1=85 --define val2=0x55 -dval3=-1 --define=val4=0xff
; output: out.txt