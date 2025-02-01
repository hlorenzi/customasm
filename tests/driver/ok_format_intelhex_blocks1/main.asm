#ruledef test
{
    halt => 0x55
}

#d64 0
halt
#d8 0
halt

#addr 0x200
#d "hello"

; command: main.asm -f intelhex -o out.txt
; output: out.txt