#include "<std>/platform/nes/cpu.asm"
#include "<std>/platform/nes/ines_nrom.asm"
#include "<std>/platform/nes/constants.asm"


#bank zeropage

zero1: #res 16
zero2: #res 16
zero3: #res 16

#bank ram

var1: #res 256
var2: #res 256
var3: #res 256

#bank prg

#addr 0x8000
reset:
#addr 0x9000
nmi:
#addr 0xa000
irq:

; command: main.asm -f mesen-mlb -o out.txt
; output: out.txt