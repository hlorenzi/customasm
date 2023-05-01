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

; command: main.asm -f annotated,base:8,group:4 -- -f symbols -o symbols.txt -- -f binary
; output: main.txt
; output: symbols.txt
; output: main.bin