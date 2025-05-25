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

; command: main.asm -f list,base:16,group:2,before:"begin\x20data\n",after:"\nend\x20data",between:"\x20" -o out.txt
; output: out.txt