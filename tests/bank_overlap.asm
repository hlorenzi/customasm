; :::

#bankdef a { #size 0x10, #outp 0x00 }
#bankdef b { #size 0x10, #outp 0x10 }
; = 0x

; :::

#bankdef a { #size 0x10, #outp 0x10 }
#bankdef b { #size 0x10, #outp 0x00 }
; = 0x

; :::

#bankdef a { #size 0x11, #outp 0x10 }
#bankdef b { #size 0x10, #outp 0x00 }
; = 0x

; :::

#bankdef a { #addr 0x20, #size 0x10, #outp 0x00 }
#bankdef b { #addr 0x20, #size 0x10, #outp 0x10 }
; = 0x

; :::

#bankdef a { #addr 0x20, #addr_end 0x30, #outp 0x00 }
#bankdef b { #addr 0x40, #addr_end 0x50, #outp 0x10 }
; = 0x

; :::

#bankdef a { #size 0x11, #outp 0x00 }
#bankdef b { #size 0x10, #outp 0x10 } ; error: overlaps

; :::

#bankdef a { #size 0x10, #outp 0x10 }
#bankdef b { #size 0x11, #outp 0x00 } ; error: overlaps

; :::

#bankdef a { #addr 0x20, #addr_end 0x31, #outp 0x00 }
#bankdef b { #addr 0x40, #addr_end 0x50, #outp 0x10 } ; error: overlaps

; :::

#bankdef a { #outp 0x00 }
#bankdef b { #outp 0x10 } ; error: overlaps