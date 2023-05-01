#ruledef test
{
    halt => 0x55
}

halt
halt
halt
halt

; command: main.asm -f annotated,base:8,group:4 -o out.txt
; output: out.txt