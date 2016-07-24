start:
	lda #0xff
	bra loop

loop:
	lda #0xabc
	bra start