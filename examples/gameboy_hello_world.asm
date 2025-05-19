#include "<std>/platform/gameboy/cpu.asm"
#include "<std>/platform/gameboy/constants.asm"
#include "<std>/platform/gameboy/cart_rom.asm"

; number of bits to align RST/interrupt vectors to
VECTOR_ALIGNMENT = 8*8

#addr INT_HANDLER_VBLANK
VBlank:
reti
#align VECTOR_ALIGNMENT
LCD:
reti
#align VECTOR_ALIGNMENT
Timer:
reti
#align VECTOR_ALIGNMENT
Serial:
reti
#align VECTOR_ALIGNMENT
Joypad:
reti
#align VECTOR_ALIGNMENT
#addr $0100 ; gameboy header starts at 0x100
; entrypoint (we get 4 bytes here, which run when the gameboy's boot ROM exits)
; this is a standard entrypoint in lots of gameboy games
nop
jp Start
; nintendo logo (must match what the boot ROM expects or we won't be able to
; boot on real hardware)
#d NINTENDO_LOGO ; constants.asm provides the correct value
; title of our ROM
Title:
#d "HELLOWORLD"
.len = $-Title
#d 0`((15-Title.len)*8)
; a byte to determine if we support GBC
; (GBC is backwards compatible with original DMG but this byte is for whether
; the game supports additional features when run on GBC)
#d CART_COMPATIBLE_DMG ; we don't
; 2-character licensee code (if $014B==0x33)
#d "CA" ; for *c*ustom*a*sm
; a byte to determine if we support SGB
; (again, this is for if we have additional features when played in a SGB)
#d CART_INDICATOR_GB ; we don't
; cartridge type
#d CART_TYPE ; cart_rom.asm set this for us
; ROM size
#d CART_ROM_32KB
; SRAM size
#d CART_SRAM_NONE
; destination code (not actually checked AFAICT)
#d CART_DEST_NON_JAPANESE
; old licensee code
#d 0x33 ; we used the new licensee code above so this must be 0x33
; version number
#d 0x00
; header checksum (must be correct to run on real hardware)
; ...we can't actually do this. customasm just isn't powerful enough to compute
; this checksum and include it
; for now you can use rgbfix from RGBDS(.github.io) to "fix" the header (pass
; -v option to just fix the header)
#d8 0
; global checksum
; same issue (though at least this one isn't checked by anything, except for
; GB Tower on Pokemon Stadium)
; rgbfix -v will fix this one too
#d16 0

#addr $0150
Start:
di                  ; disable interrupts
ld sp, $dfff        ; initialize stack pointer
ld a, %11100100     ; initialize palette
ld [rBGP], a
xor a               ; zero out scroll x and y
ld [rSCX], a
ld [rSCY], a
; turn off LCD
ld a, [rLCDC]
rlca                ; LCD on/off flag in carry
jr nc, .skip        ; LCD already off? skip turning it off then
.waitForLY:
ld a, [rLY]
cp 145              ; line 145 is start of vblank
jr nz, .waitForLY   ; if we aren't there yet, loop
ld a, [rLCDC]
res 7, a            ; reset on/off flag
ld [rLCDC], a
.skip:
ld hl, Tiles        ; load font into character RAM
ld de, _VRAM
ld bc, Tiles.size
inc b
.tileLoop:
ld a,[hli]
ld [de],a
inc de
ld [de],a
inc de
dec c
jr nz, .tileLoop
dec b
jr nz, .tileLoop
; clear screen
ld hl, _SCRN0
ld bc, SCRN_VX_B * SCRN_VY_B
ld a, " "
inc b
.clearScreenLoop:
ld [hli], a
dec c
jr nz, .clearScreenLoop
dec b
jr nz, .clearScreenLoop
; now write hello world in the center
ld hl, HelloWorldFrom
ld de, _SCRN0+(8*SCRN_VX_B)+2
ld c, HelloWorldFrom.size
.helloWorldLoop:
ld a,[hli]
ld [de],a
inc de
dec c
jr nz, .helloWorldLoop
ld hl, Customasm
ld de, _SCRN0+(9*SCRN_VX_B)+2
ld c, Customasm.size
.helloWorldLoop2:
ld a,[hli]
ld [de],a
inc de
dec c
jr nz, .helloWorldLoop2
; turn the LCD back on and wait
ld a, LCDCF_ON | LCDCF_BLK01 | LCDCF_BG9800 | LCDCF_BGON | LCDCF_OBJOFF
ld [rLCDC], a
ei
.spin:
halt
nop
jr .spin

HelloWorldFrom:
#d "Hello world from"
.size = $ - HelloWorldFrom
Customasm:
#d "   customasm!"
.size = $ - Customasm

Tiles:
#d incbin("ibmpc1.bin")
.size = $ - Tiles