#ruledef
{
    nop => 0x00
    ldi sp, {value: i8} => 0x08 @ value
}

nop
ldi sp, 0xff
nop

; command: main.asm -f intelhex -o out.txt
; output: out.txt