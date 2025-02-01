#ruledef test
{
    halt => 0x55
}

#addr 0x100

#d64 0
halt
#d8 0
halt

#addr 0x200
label1:
#d "hello"
#d256 0

#addr 0x400
#d "world"
label2:
#d64 0

#addr 0x600
label3:

; command: main.asm -f intelhex -o out.txt
; output: out.txt