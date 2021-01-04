; ::: cpu.asm

#ruledef test
{
    halt => 0x55
}

; ::: code.asm

halt
halt
halt
#d8 0xaa, 0x55, 0xaa

; ::: code2.asm

#include "code.asm"



; :::

#include "cpu.asm"
halt ; = 0x55

; :::

#include "cpu.asm"
#include "code.asm"
; = 0x555555aa55aa

; :::

#include "cpu.asm"
#include "code2.asm"
; = 0x555555aa55aa

; :::

#include "code.asm"
; error: code.asm:2: no match
; error: code.asm:3: no match
; error: code.asm:4: no match

; :::

#include "code.asm"
#include "code2.asm"
; error: code.asm:2: no match
; error: code.asm:3: no match
; error: code.asm:4: no match

; :::

halt ; error: no match
#include "code.asm"
halt ; error: no match
; error: code.asm:2: no match
; error: code.asm:3: no match
; error: code.asm:4: no match

; :::

#include "unk.asm" ; error: not found

; :::

#include "/unk.asm" ; error: not found

; failing in GitHub actions:
;
;#include "C:/unk.asm" ; invalid