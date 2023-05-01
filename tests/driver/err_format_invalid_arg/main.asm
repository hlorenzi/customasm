#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -f annotated,group:2,unknown:32 -o out.txt
; error: unknown format argument `annotated,unknown`