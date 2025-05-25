#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -f annotated,unknown1:64,group:2,unknown2:32 -o out.txt
; error: unknown format argument `annotated,unknown1`
; error: unknown format argument `annotated,unknown2`