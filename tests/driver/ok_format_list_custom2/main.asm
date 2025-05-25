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

; command: main.asm -f list,base:16,group:2,group2:16,before:"begin\n\t",after:"\nend",between:"\x20",between2:"\n\t" -o out.txt
; output: out.txt