#include "cpu6502.asm"
#include "ines_nrom.asm"
#include "nes.asm"


#bank zeropage

varTimer: #res 1
varPaletteIndex: #res 1


#bank prg

reset:
	; disable IRQs and decimal mode
	sei
	cld
	
	; disable APU frame IRQ
	ldx #0x40
	stx APU_FRMCNTR
	
	; set up stack
	ldx #0xff
	txs
	
	; disable NMI	
	inx
	stx PPU_CTRL
	
	; disable rendering
	stx PPU_MASK
	
	; disable DMC IRQs
	stx APU_DMC

	; wait for PPU to be ready
	.vblankwait1:
		bit PPU_STATUS
		bpl .vblankwait1
	
	; clear memory
	.clearmem:
		lda #0x00
		sta 0x0000, x
		sta 0x0100, x
		sta 0x0200, x
		sta 0x0300, x
		sta 0x0400, x
		sta 0x0500, x
		sta 0x0600, x
		sta 0x0700, x
		inx
		bne .clearmem
		
	; wait for PPU to be ready again
	.vblankwait2:
		bit PPU_STATUS
		bpl .vblankwait2
		
	; enable rendering
	lda #PPU_MASK_SHOWBKG | PPU_MASK_LEFTBKG
	sta PPU_MASK
	
	; enable NMI
	lda #PPU_CTRL_NMI
	sta PPU_CTRL
	
	; wait for NMI
	.infinite:
		jmp .infinite
	
	
nmi:
	; increment timer
	inc varTimer
	lda varTimer
	cmp #8
	bne .end
	
	; if timer reached 8...
		lda #0
		sta varTimer
		
		; update background color
		lda PPU_STATUS
		
		lda #VRAM_PALETTE[15:8]
		sta PPU_ADDR
		lda #VRAM_PALETTE[7:0]
		sta PPU_ADDR
		
		ldx varPaletteIndex
		lda palette, x
		sta PPU_DATA
		
		; increment palette index
		inc varPaletteIndex
		lda varPaletteIndex
		cmp #paletteLen
		bne .end
		
		; if pallete index reached the end of the table...
			lda #0
			sta varPaletteIndex

.end:
irq:
	rti
	
	
palette:
	#d8 0x0d, 0x01, 0x12, 0x21, 0x31, 0x21, 0x12, 0x01, 0x0d ; blues
	#d8 0x0d, 0x06, 0x16, 0x26, 0x36, 0x26, 0x16, 0x06, 0x0d ; reds
	#d8 0x0d, 0x09, 0x19, 0x29, 0x39, 0x29, 0x19, 0x09, 0x0d ; greens

paletteLen = $ - palette