#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld val ; error: failed / note:_:3: within / error: unknown symbol `val`

; command: main.asm -o out.bin -dval