#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#bankdef test { #bits 3, #addr 0, #outp 0 }

#d3 $ ; = 0b000
#res 2 ; = 0b000_000
label:
#res 3 ; = 0b000_000_000
#d3 $ ; = 0b110
#d3 label ; = 0b011