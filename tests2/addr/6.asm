#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#bankdef test { #bits 3, #addr 0, #outp 0 }

#d3 $ ; = 0b000
#addr 0b010 ; = 0b000
label:
#addr 0b100 ; = 0b000_000
#d3 $ ; = 0b100
#d3 label ; = 0b010