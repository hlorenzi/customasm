#ruledef test
{
    halt => 0x55
}

halt
halt

; command: main.asm -t x
; error: invalid argument for `--iters`