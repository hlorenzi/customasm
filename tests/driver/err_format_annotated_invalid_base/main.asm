#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -f annotated,group:4,base:3 -o out.txt
; error: invalid format argument `annotated,base