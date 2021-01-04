; ::: cpu/cpu.asm

#ruledef test
{
    halt => 0x55
}

; ::: code/code.asm

halt

; ::: code/code2.asm

#include "code.asm"

; ::: code3.asm

#include "code/code.asm"

; ::: code/code4.asm

#include "../code3.asm"

; ::: code/code5.asm

#include "/code3.asm"



; :::

#include "cpu/cpu.asm"
#include "code/code.asm"
; = 0x55

; :::

#include "./cpu/./cpu.asm"
#include "././code///code.asm"
#include "/code/code.asm/"
#include "code/../unk/../code/unk/unk/../../code.asm"
#include ".\\code\\\\code.asm"
; = 0x55

; :::

#include "cpu/cpu.asm"
#include "code/code2.asm"
; = 0x55

; :::

#include "cpu/cpu.asm"
#include "code/code2.asm"
; = 0x55

; :::

#include "cpu/cpu.asm"
#include "code3.asm"
; = 0x55

; :::

#include "cpu/cpu.asm"
#include "code/code4.asm"
; = 0x55

; :::

#include "cpu/cpu.asm"
#include "code/code5.asm"
; = 0x55

; :::

#include "../cpu.asm" ; error: out of project directory