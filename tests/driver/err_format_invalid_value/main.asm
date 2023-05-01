#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -f annotated,group:2,base:8:16 -o out.txt
; error: invalid format argument `annotated,base