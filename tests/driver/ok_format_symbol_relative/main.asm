#ruledef test
{
    halt => 0x55
}

start:
halt
loop:
halt
.inner:
halt
..inner2:
halt
end:
halt

; command: main.asm -f relative-symbols -o out.txt
; output: out.txt