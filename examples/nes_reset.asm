#addr 0x8000


PPU_CTRL    = 0x2000
PPU_MASK    = 0x2001
PPU_STATUS  = 0x2002
APU_FRMCNTR = 0x4017


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
	stx 0x4010

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
	
; infinite loop
.infinite:
	jmp .infinite