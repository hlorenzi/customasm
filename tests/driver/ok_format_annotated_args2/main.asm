#ruledef test
{
    halt => 0x55
}

label1:
halt
halt
label2:
halt
halt

; command: main.asm -f annotated,base:2,addr_base:2,group:4,labels:false -o out.txt
; output: out.txt