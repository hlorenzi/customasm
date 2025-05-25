#ruledef test
{
    halt => 0x55
}

halt
halt
#d "hello, world!"
#d "hello, world!"
#d "hello, world!"
#d "hello, world!"

; command: main.asm -f list,base:16,between:20 -o out.txt
; error: invalid format argument `list,between