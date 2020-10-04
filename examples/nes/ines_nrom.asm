#bankdef header   { #addr 0x0,    #size 0x10,   #outp 8 * 0x0    }
#bankdef prg      { #addr 0x8000, #size 0x7ffa, #outp 8 * 0x10   }
#bankdef vectors  { #addr 0xfffa, #size 0x6,    #outp 8 * 0x800a }
#bankdef zeropage { #addr 0x0,    #size 0x100 }
#bankdef ram      { #addr 0x200,  #size 0x600 }


#bank header

; magic number
#d "NES", 0x1a

#d8 2 ; 16KB PRG bank count
#d8 0 ; 8KB CHR bank count
#d4 0 ; low nybble of mapper id
#d1 0
#d1 0 ; trainer presence
#d1 0 ; SRAM presence
#d1 0 ; mirroring
#d4 0 ; high nybble of mapper id
#d4 0
#d8 0
#d8 0
#d2 0
#d1 0 ; bus conflict presence
#d1 0 ; extra RAM presence
#d2 0
#d2 0 ; region


#bank vectors
#d16   nmi[7:0] @   nmi[15:8]
#d16 reset[7:0] @ reset[15:8]
#d16   irq[7:0] @   irq[15:8]