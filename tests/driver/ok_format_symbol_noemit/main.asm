#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

#fn add1(x) => x + 1

start:
x1 = add1(0x10)
ld x1

#const x2 = 0x11
ld x2

#const(noemit) x3 = 0x11
ld x3

loop:
#const y1 = 0x22
ld y1

#const y2 = 0x22
ld y2

#const(noemit) y3 = 0x22
ld y3

.inner:
.z1 = 0x33
ld .z1

#const .z2 = 0x33
ld .z2

#const(noemit) .z3 = 0x33
ld .z3

; command: main.asm -f symbols -o out.txt
; output: out.txt