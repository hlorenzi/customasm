#ruledef test
{
    halt => 0x55
}

start:
halt
loop:
halt
.inner:
halt

; command: main.asm --symbol-format default -s out.txt
; output: out.txt