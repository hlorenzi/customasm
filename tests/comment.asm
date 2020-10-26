; :::
; = 0x00
#d8 0, ; 1, 2, 3

; :::
; = 0x000203
#d8 0, ;* 1, *; 2, 3

; :::
; = 0x000203
#d8 0 ;*
  #d8 1, comment
*;
#d8 2, 3

; :::
; = 0x0003
#d8 0 ;*
  ;* #d8 1, comment
*;
#d8 2, comment *;
#d8 3

; :::
#d8 0 ;* ; error: expected line break
  ;* #d8 1, comment
*;
#d8 2, comment *; #d8 3

; :::
; = 0x00
#d8 0 ;**;

; :::
#d8 0 ;** ; error: unexpected

; :::
#d8 0 ;* ; error: unexpected