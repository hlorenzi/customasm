#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -t 0
; error: invalid argument for `--iters`