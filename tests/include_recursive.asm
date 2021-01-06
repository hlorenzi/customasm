; ::: recursive1.asm
#include "recursive1.asm"

; ::: recursive2.asm
#include "recursive3.asm"

; ::: recursive3.asm
#include "recursive2.asm"

; ::: recursive4.asm
#include "recursive2.asm"


; :::
#include "recursive1.asm"
; error: recursive1.asm:1: recursive

; :::
#include "recursive2.asm"
; error: recursive3.asm:1: recursive

; :::
#include "recursive4.asm"
; error: recursive3.asm:1: recursive