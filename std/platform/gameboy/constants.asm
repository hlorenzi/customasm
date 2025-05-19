#once
; *
; * Game Boy Hardware definitions
; * https://github.com/gbdev/hardware.inc
; *
; * Based on Jones' hardware.inc
; * And based on Carsten Sorensen's ideas.
; *
; * To the extent possible under law, the authors of this work have
; * waived all copyright and related or neighboring rights to the work.
; * See https://creativecommons.org/publicdomain/zero/1.0/ for details.
; *
; * SPDX-License-Identifier: CC0-1.0
; *
; * Rev 1.1 - 15-Jul-97 : Added define check
; * Rev 1.2 - 18-Jul-97 : Added revision check macro
; * Rev 1.3 - 19-Jul-97 : Modified for RGBASM V1.05
; * Rev 1.4 - 27-Jul-97 : Modified for new subroutine prefixes
; * Rev 1.5 - 15-Aug-97 : Added _HRAM, PAD, CART defines
; *                     :  and Nintendo Logo
; * Rev 1.6 - 30-Nov-97 : Added rDIV, rTIMA, rTMA, & rTAC
; * Rev 1.7 - 31-Jan-98 : Added _SCRN0, _SCRN1
; * Rev 1.8 - 15-Feb-98 : Added rSB, rSC
; * Rev 1.9 - 16-Feb-98 : Converted I/O registers to $FFXX format
; * Rev 2.0 -           : Added GBC registers
; * Rev 2.1 -           : Added MBC5 & cart RAM enable/disable defines
; * Rev 2.2 -           : Fixed NR42,NR43, & NR44 equates
; * Rev 2.3 -           : Fixed incorrect _HRAM equate
; * Rev 2.4 - 27-Apr-13 : Added some cart defines (AntonioND)
; * Rev 2.5 - 03-May-15 : Fixed format (AntonioND)
; * Rev 2.6 - 09-Apr-16 : Added GBC OAM and cart defines (AntonioND)
; * Rev 2.7 - 19-Jan-19 : Added rPCMXX (ISSOtm)
; * Rev 2.8 - 03-Feb-19 : Added audio registers flags (Álvaro Cuesta)
; * Rev 2.9 - 28-Feb-20 : Added utility rP1 constants
; * Rev 3.0 - 27-Aug-20 : Register ordering, byte-based sizes, OAM additions, general cleanup (Blitter Object)
; * Rev 4.0 - 03-May-21 : Updated to use RGBDS 0.5.0 syntax, changed IEF_LCDC to IEF_STAT (Eievui)
; * Rev 4.1 - 16-Aug-21 : Added more flags, bit number defines, and offset constants for OAM and window positions (rondnelson99)
; * Rev 4.2 - 04-Sep-21 : Added CH3- and CH4-specific audio registers flags (ISSOtm)
; * Rev 4.3 - 07-Nov-21 : Deprecate VRAM address constants (Eievui)
; * Rev 4.4 - 11-Jan-22 : Deprecate VRAM CART_SRAM_2KB constant (avivace)
; * Rev 4.5 - 03-Mar-22 : Added bit number definitions for OCPS, BCPS and LCDC (sukus)
; * Rev 4.6 - 15-Jun-22 : Added MBC3 registers and special values
; * Rev 4.7.0 - 27-Jun-22 : Added alternate names for some constants
; * Rev 4.7.1 - 05-Jul-22 : Added RPB_LED_ON constant
; * Rev 4.8.0 - 25-Oct-22 : Changed background addressing constants (zlago)
; * Rev 4.8.1 - 29-Apr-23 : Added rOPRI (rbong)
; * Rev 4.9.0 - 24-Jun-23 : Added definitions for interrupt vectors (sukus)
; * Rev 4.9.1 - 11-Sep-23 : Added repository link and CC0 waiver notice
; * Rev 4.9.2 - 18-Aug-24 : Corrected CART_ROM_MBC5_BAT to CART_ROM_MBC5_RAM (DevEd)
; *
; * Initial conversion to customasm format by MineRobber9000 19-May-25
; * Check back with upstream every now and again and port changes (not that I expect there to be many changes)
; * I also removed the deprecated constants, since I don't need to maintain backcompatibility with over 25 years of people using
; * hardware.inc in their RGBDS projects.


; ***************************************************************************
; *
; * General memory region constants
; *
; ***************************************************************************

_VRAM        = $8000 ; $8000->$9FFF
_SCRN0       = $9800 ; $9800->$9BFF
_SCRN1       = $9C00 ; $9C00->$9FFF
_SRAM        = $A000 ; $A000->$BFFF
_RAM         = $C000 ; $C000->$CFFF / $C000->$DFFF
_RAMBANK     = $D000 ; $D000->$DFFF
_OAMRAM      = $FE00 ; $FE00->$FE9F
_IO          = $FF00 ; $FF00->$FF7F,$FFFF
_AUD3WAVERAM = $FF30 ; $FF30->$FF3F
_HRAM        = $FF80 ; $FF80->$FFFE


; ***************************************************************************
; *
; * MBC registers
; *
; ***************************************************************************

; *** Common ***

; --
; -- RAMG ($0000-$1FFF)
; -- Controls whether access to SRAM (and the MBC3 RTC registers) is allowed (W)
; --
rRAMG = $0000

CART_SRAM_ENABLE  = $0A
CART_SRAM_DISABLE = $00


; --
; -- ROMB0 ($2000-$3FFF)
; -- Selects which ROM bank is mapped to the ROMX space ($4000-$7FFF) (W)
; --
; -- The range of accepted values, as well as the behavior of writing $00,
; -- varies depending on the MBC.
; --
rROMB0 = $2000

; --
; -- RAMB ($4000-$5FFF)
; -- Selects which SRAM bank is mapped to the SRAM space ($A000-$BFFF) (W)
; --
; -- The range of accepted values varies depending on the cartridge configuration.
; --
rRAMB = $4000


; *** MBC3-specific registers ***

; Write one of these to rRAMG to map the corresponding RTC register to all SRAM space
RTC_S  = $08 ; Seconds  (0-59)
RTC_M  = $09 ; Minutes  (0-59)
RTC_H  = $0A ; Hours    (0-23)
RTC_DL = $0B ; Lower 8 bits of Day Counter ($00-$FF)
RTC_DH = $0C ; Bit 7 - Day Counter Carry Bit (1=Counter Overflow)
                   ; Bit 6 - Halt (0=Active, 1=Stop Timer)
                   ; Bit 0 - Most significant bit of Day Counter (Bit 8)


; --
; -- RTCLATCH ($6000-$7FFF)
; -- Write $00 then $01 to latch the current time into the RTC registers (W)
; --
rRTCLATCH = $6000


; *** MBC5-specific register ***

; --
; -- ROMB1 ($3000-$3FFF)
; -- A 9th bit that "extends" ROMB0 if more than 256 banks are present (W)
; --
; -- Also note that rROMB0 thus only spans $2000-$2FFF.
; --
rROMB1 = $3000


; Bit 3 of RAMB enables the rumble motor (if any)
CART_RUMBLE_ON = 1 << 3


; ***************************************************************************
; *
; * Memory-mapped registers
; *
; ***************************************************************************

; --
; -- P1 ($FF00)
; -- Register for reading joy pad info. (R/W)
; --
rP1 = $FF00

P1F_5 = %00100000 ; P15 out port, set to 0 to get buttons
P1F_4 = %00010000 ; P14 out port, set to 0 to get dpad
P1F_3 = %00001000 ; P13 in port
P1F_2 = %00000100 ; P12 in port
P1F_1 = %00000010 ; P11 in port
P1F_0 = %00000001 ; P10 in port

P1F_GET_DPAD = P1F_5
P1F_GET_BTN  = P1F_4
P1F_GET_NONE = P1F_4 | P1F_5


; --
; -- SB ($FF01)
; -- Serial Transfer Data (R/W)
; --
rSB = $FF01


; --
; -- SC ($FF02)
; -- Serial I/O Control (R/W)
; --
rSC = $FF02

SCF_START  = %10000000 ; Transfer Start Flag (1=Transfer in progress, or requested)
SCF_SPEED  = %00000010 ; Clock Speed (0=Normal, 1=Fast) ** CGB Mode Only **
SCF_SOURCE = %00000001 ; Shift Clock (0=External Clock, 1=Internal Clock)

SCB_START  = 7
SCB_SPEED  = 1
SCB_SOURCE = 0

; --
; -- DIV ($FF04)
; -- Divider register (R/W)
; --
rDIV = $FF04


; --
; -- TIMA ($FF05)
; -- Timer counter (R/W)
; --
rTIMA = $FF05


; --
; -- TMA ($FF06)
; -- Timer modulo (R/W)
; --
rTMA = $FF06


; --
; -- TAC ($FF07)
; -- Timer control (R/W)
; --
rTAC = $FF07

TACF_START  = %00000100
TACF_STOP   = %00000000
TACF_4KHZ   = %00000000
TACF_16KHZ  = %00000011
TACF_65KHZ  = %00000010
TACF_262KHZ = %00000001

TACB_START  = 2


; --
; -- IF ($FF0F)
; -- Interrupt Flag (R/W)
; --
rIF = $FF0F


; --
; -- AUD1SWEEP/NR10 ($FF10)
; -- Sweep register (R/W)
; --
; -- Bit 6-4 - Sweep Time
; -- Bit 3   - Sweep Increase/Decrease
; --           0: Addition    (frequency increases???)
; --           1: Subtraction (frequency increases???)
; -- Bit 2-0 - Number of sweep shift (# 0-7)
; -- Sweep Time: (n*7.8ms)
; --
rNR10 = $FF10
rAUD1SWEEP = rNR10

AUD1SWEEP_UP   = %00000000
AUD1SWEEP_DOWN = %00001000


; --
; -- AUD1LEN/NR11 ($FF11)
; -- Sound length/Wave pattern duty (R/W)
; --
; -- Bit 7-6 - Wave Pattern Duty (00:12.5% 01:25% 10:50% 11:75%)
; -- Bit 5-0 - Sound length data (# 0-63)
; --
rNR11 = $FF11
rAUD1LEN = rNR11


; --
; -- AUD1ENV/NR12 ($FF12)
; -- Envelope (R/W)
; --
; -- Bit 7-4 - Initial value of envelope
; -- Bit 3   - Envelope UP/DOWN
; --           0: Decrease
; --           1: Range of increase
; -- Bit 2-0 - Number of envelope sweep (# 0-7)
; --
rNR12 = $FF12
rAUD1ENV = rNR12


; --
; -- AUD1LOW/NR13 ($FF13)
; -- Frequency low byte (W)
; --
rNR13 = $FF13
rAUD1LOW = rNR13


; --
; -- AUD1HIGH/NR14 ($FF14)
; -- Frequency high byte (W)
; --
; -- Bit 7   - Initial (when set, sound restarts)
; -- Bit 6   - Counter/consecutive selection
; -- Bit 2-0 - Frequency's higher 3 bits
; --
rNR14 = $FF14
rAUD1HIGH = rNR14


; --
; -- AUD2LEN/NR21 ($FF16)
; -- Sound Length; Wave Pattern Duty (R/W)
; --
; -- see AUD1LEN for info
; --
rNR21 = $FF16
rAUD2LEN = rNR21


; --
; -- AUD2ENV/NR22 ($FF17)
; -- Envelope (R/W)
; --
; -- see AUD1ENV for info
; --
rNR22 = $FF17
rAUD2ENV = rNR22


; --
; -- AUD2LOW/NR23 ($FF18)
; -- Frequency low byte (W)
; --
rNR23 = $FF18
rAUD2LOW = rNR23


; --
; -- AUD2HIGH/NR24 ($FF19)
; -- Frequency high byte (W)
; --
; -- see AUD1HIGH for info
; --
rNR24 = $FF19
rAUD2HIGH = rNR24


; --
; -- AUD3ENA/NR30 ($FF1A)
; -- Sound on/off (R/W)
; --
; -- Bit 7   - Sound ON/OFF (1=ON,0=OFF)
; --
rNR30 = $FF1A
rAUD3ENA = rNR30

AUD3ENA_OFF = %00000000
AUD3ENA_ON  = %10000000


; --
; -- AUD3LEN/NR31 ($FF1B)
; -- Sound length (R/W)
; --
; -- Bit 7-0 - Sound length
; --
rNR31 = $FF1B
rAUD3LEN = rNR31


; --
; -- AUD3LEVEL/NR32 ($FF1C)
; -- Select output level
; --
; -- Bit 6-5 - Select output level
; --           00: 0/1 (mute)
; --           01: 1/1
; --           10: 1/2
; --           11: 1/4
; --
rNR32 = $FF1C
rAUD3LEVEL = rNR32

AUD3LEVEL_MUTE = %00000000
AUD3LEVEL_100  = %00100000
AUD3LEVEL_50   = %01000000
AUD3LEVEL_25   = %01100000


; --
; -- AUD3LOW/NR33 ($FF1D)
; -- Frequency low byte (W)
; --
; -- see AUD1LOW for info
; --
rNR33 = $FF1D
rAUD3LOW = rNR33


; --
; -- AUD3HIGH/NR34 ($FF1E)
; -- Frequency high byte (W)
; --
; -- see AUD1HIGH for info
; --
rNR34 = $FF1E
rAUD3HIGH = rNR34


; --
; -- AUD4LEN/NR41 ($FF20)
; -- Sound length (R/W)
; --
; -- Bit 5-0 - Sound length data (# 0-63)
; --
rNR41 = $FF20
rAUD4LEN = rNR41


; --
; -- AUD4ENV/NR42 ($FF21)
; -- Envelope (R/W)
; --
; -- see AUD1ENV for info
; --
rNR42 = $FF21
rAUD4ENV = rNR42


; --
; -- AUD4POLY/NR43 ($FF22)
; -- Polynomial counter (R/W)
; --
; -- Bit 7-4 - Selection of the shift clock frequency of the (scf)
; --           polynomial counter (0000-1101)
; --           freq=drf*1/2^scf (not sure)
; -- Bit 3 -   Selection of the polynomial counter's step
; --           0: 15 steps
; --           1: 7 steps
; -- Bit 2-0 - Selection of the dividing ratio of frequencies (drf)
; --           000: f/4   001: f/8   010: f/16  011: f/24
; --           100: f/32  101: f/40  110: f/48  111: f/56  (f=4.194304 Mhz)
; --
rNR43 = $FF22
rAUD4POLY = rNR43

AUD4POLY_15STEP = %00000000
AUD4POLY_7STEP  = %00001000


; --
; -- AUD4GO/NR44 ($FF23)
; --
; -- Bit 7 -   Initial (when set, sound restarts)
; -- Bit 6 -   Counter/consecutive selection
; --
rNR44 = $FF23
rAUD4GO = rNR44


; --
; -- AUDVOL/NR50 ($FF24)
; -- Channel control / ON-OFF / Volume (R/W)
; --
; -- Bit 7   - Vin->SO2 ON/OFF (left)
; -- Bit 6-4 - SO2 output level (left speaker) (# 0-7)
; -- Bit 3   - Vin->SO1 ON/OFF (right)
; -- Bit 2-0 - SO1 output level (right speaker) (# 0-7)
; --
rNR50 = $FF24
rAUDVOL = rNR50

AUDVOL_VIN_LEFT  = %10000000 ; SO2
AUDVOL_VIN_RIGHT = %00001000 ; SO1


; --
; -- AUDTERM/NR51 ($FF25)
; -- Selection of Sound output terminal (R/W)
; --
; -- Bit 7   - Output channel 4 to SO2 terminal (left)
; -- Bit 6   - Output channel 3 to SO2 terminal (left)
; -- Bit 5   - Output channel 2 to SO2 terminal (left)
; -- Bit 4   - Output channel 1 to SO2 terminal (left)
; -- Bit 3   - Output channel 4 to SO1 terminal (right)
; -- Bit 2   - Output channel 3 to SO1 terminal (right)
; -- Bit 1   - Output channel 2 to SO1 terminal (right)
; -- Bit 0   - Output channel 1 to SO1 terminal (right)
; --
rNR51 = $FF25
rAUDTERM = rNR51

; SO2
AUDTERM_4_LEFT  = %10000000
AUDTERM_3_LEFT  = %01000000
AUDTERM_2_LEFT  = %00100000
AUDTERM_1_LEFT  = %00010000
; SO1
AUDTERM_4_RIGHT = %00001000
AUDTERM_3_RIGHT = %00000100
AUDTERM_2_RIGHT = %00000010
AUDTERM_1_RIGHT = %00000001


; --
; -- AUDENA/NR52 ($FF26)
; -- Sound on/off (R/W)
; --
; -- Bit 7   - All sound on/off (sets all audio regs to 0!)
; -- Bit 3   - Sound 4 ON flag (read only)
; -- Bit 2   - Sound 3 ON flag (read only)
; -- Bit 1   - Sound 2 ON flag (read only)
; -- Bit 0   - Sound 1 ON flag (read only)
; --
rNR52 = $FF26
rAUDENA = rNR52

AUDENA_ON    = %10000000
AUDENA_OFF   = %00000000  ; sets all audio regs to 0!


; --
; -- LCDC ($FF40)
; -- LCD Control (R/W)
; --
rLCDC = $FF40

LCDCF_OFF     = %00000000 ; LCD Control Operation
LCDCF_ON      = %10000000 ; LCD Control Operation
LCDCF_WIN9800 = %00000000 ; Window Tile Map Display Select
LCDCF_WIN9C00 = %01000000 ; Window Tile Map Display Select
LCDCF_WINOFF  = %00000000 ; Window Display
LCDCF_WINON   = %00100000 ; Window Display
LCDCF_BLK21   = %00000000 ; BG & Window Tile Data Select
LCDCF_BLK01   = %00010000 ; BG & Window Tile Data Select
LCDCF_BG9800  = %00000000 ; BG Tile Map Display Select
LCDCF_BG9C00  = %00001000 ; BG Tile Map Display Select
LCDCF_OBJ8    = %00000000 ; OBJ Construction
LCDCF_OBJ16   = %00000100 ; OBJ Construction
LCDCF_OBJOFF  = %00000000 ; OBJ Display
LCDCF_OBJON   = %00000010 ; OBJ Display
LCDCF_BGOFF   = %00000000 ; BG Display
LCDCF_BGON    = %00000001 ; BG Display

LCDCB_ON      = 7 ; LCD Control Operation
LCDCB_WIN9C00 = 6 ; Window Tile Map Display Select
LCDCB_WINON   = 5 ; Window Display
LCDCB_BLKS    = 4 ; BG & Window Tile Data Select
LCDCB_BG9C00  = 3 ; BG Tile Map Display Select
LCDCB_OBJ16   = 2 ; OBJ Construction
LCDCB_OBJON   = 1 ; OBJ Display
LCDCB_BGON    = 0 ; BG Display
; "Window Character Data Select" follows BG


; --
; -- STAT ($FF41)
; -- LCDC Status   (R/W)
; --
rSTAT = $FF41

STATF_LYC     =  %01000000 ; LYC=LY Coincidence (Selectable)
STATF_MODE10  =  %00100000 ; Mode 10
STATF_MODE01  =  %00010000 ; Mode 01 (V-Blank)
STATF_MODE00  =  %00001000 ; Mode 00 (H-Blank)
STATF_LYCF    =  %00000100 ; Coincidence Flag
STATF_HBL     =  %00000000 ; H-Blank
STATF_VBL     =  %00000001 ; V-Blank
STATF_OAM     =  %00000010 ; OAM-RAM is used by system
STATF_LCD     =  %00000011 ; Both OAM and VRAM used by system
STATF_BUSY    =  %00000010 ; When set, VRAM access is unsafe

STATB_LYC     =  6
STATB_MODE10  =  5
STATB_MODE01  =  4
STATB_MODE00  =  3
STATB_LYCF    =  2
STATB_BUSY    =  1

; --
; -- SCY ($FF42)
; -- Scroll Y (R/W)
; --
rSCY = $FF42


; --
; -- SCX ($FF43)
; -- Scroll X (R/W)
; --
rSCX = $FF43


; --
; -- LY ($FF44)
; -- LCDC Y-Coordinate (R)
; --
; -- Values range from 0->153. 144->153 is the VBlank period.
; --
rLY = $FF44


; --
; -- LYC ($FF45)
; -- LY Compare (R/W)
; --
; -- When LY==LYC, STATF_LYCF will be set in STAT
; --
rLYC = $FF45


; --
; -- DMA ($FF46)
; -- DMA Transfer and Start Address (W)
; --
rDMA = $FF46


; --
; -- BGP ($FF47)
; -- BG Palette Data (W)
; --
; -- Bit 7-6 - Intensity for %11
; -- Bit 5-4 - Intensity for %10
; -- Bit 3-2 - Intensity for %01
; -- Bit 1-0 - Intensity for %00
; --
rBGP = $FF47


; --
; -- OBP0 ($FF48)
; -- Object Palette 0 Data (W)
; --
; -- See BGP for info
; --
rOBP0 = $FF48


; --
; -- OBP1 ($FF49)
; -- Object Palette 1 Data (W)
; --
; -- See BGP for info
; --
rOBP1 = $FF49


; --
; -- WY ($FF4A)
; -- Window Y Position (R/W)
; --
; -- 0 <= WY <= 143
; -- When WY = 0, the window is displayed from the top edge of the LCD screen.
; --
rWY = $FF4A


; --
; -- WX ($FF4B)
; -- Window X Position (R/W)
; --
; -- 7 <= WX <= 166
; -- When WX = 7, the window is displayed from the left edge of the LCD screen.
; -- Values of 0-6 and 166 are unreliable due to hardware bugs.
; --
rWX = $FF4B

WX_OFS = 7 ; add this to a screen position to get a WX position


; --
; -- SPEED ($FF4D)
; -- Select CPU Speed (R/W)
; --
rKEY1 = $FF4D
rSPD  = rKEY1

KEY1F_DBLSPEED = %10000000 ; 0=Normal Speed, 1=Double Speed (R)
KEY1F_PREPARE  = %00000001 ; 0=No, 1=Prepare (R/W)


; --
; -- VBK ($FF4F)
; -- Select Video RAM Bank (R/W)
; --
; -- Bit 0 - Bank Specification (0: Specify Bank 0; 1: Specify Bank 1)
; --
rVBK = $FF4F


; --
; -- HDMA1 ($FF51)
; -- High byte for Horizontal Blanking/General Purpose DMA source address (W)
; -- CGB Mode Only
; --
rHDMA1 = $FF51


; --
; -- HDMA2 ($FF52)
; -- Low byte for Horizontal Blanking/General Purpose DMA source address (W)
; -- CGB Mode Only
; --
rHDMA2 = $FF52


; --
; -- HDMA3 ($FF53)
; -- High byte for Horizontal Blanking/General Purpose DMA destination address (W)
; -- CGB Mode Only
; --
rHDMA3 = $FF53


; --
; -- HDMA4 ($FF54)
; -- Low byte for Horizontal Blanking/General Purpose DMA destination address (W)
; -- CGB Mode Only
; --
rHDMA4 = $FF54


; --
; -- HDMA5 ($FF55)
; -- Transfer length (in tiles minus 1)/mode/start for Horizontal Blanking, General Purpose DMA (R/W)
; -- CGB Mode Only
; --
rHDMA5 = $FF55

HDMA5F_MODE_GP  = %00000000 ; General Purpose DMA (W)
HDMA5F_MODE_HBL = %10000000 ; HBlank DMA (W)
HDMA5B_MODE = 7 ; DMA mode select (W)

; -- Once DMA has started, use HDMA5F_BUSY to check when the transfer is complete
HDMA5F_BUSY = %10000000 ; 0=Busy (DMA still in progress), 1=Transfer complete (R)


; --
; -- RP ($FF56)
; -- Infrared Communications Port (R/W)
; -- CGB Mode Only
; --
rRP = $FF56

RPF_ENREAD   = %11000000
RPF_DATAIN   = %00000010 ; 0=Receiving IR Signal, 1=Normal
RPF_WRITE_HI = %00000001
RPF_WRITE_LO = %00000000

RPB_LED_ON   = 0
RPB_DATAIN   = 1


; --
; -- BCPS/BGPI ($FF68)
; -- Background Color Palette Specification (aka Background Palette Index) (R/W)
; --
rBCPS = $FF68
rBGPI = rBCPS

BCPSF_AUTOINC = %10000000 ; Auto Increment (0=Disabled, 1=Increment after Writing)
BCPSB_AUTOINC = 7
BGPIF_AUTOINC = BCPSF_AUTOINC
BGPIB_AUTOINC = BCPSB_AUTOINC


; --
; -- BCPD/BGPD ($FF69)
; -- Background Color Palette Data (aka Background Palette Data) (R/W)
; --
rBCPD = $FF69
rBGPD = rBCPD


; --
; -- OCPS/OBPI ($FF6A)
; -- Object Color Palette Specification (aka Object Background Palette Index) (R/W)
; --
rOCPS = $FF6A
rOBPI = rOCPS

OCPSF_AUTOINC = %10000000 ; Auto Increment (0=Disabled, 1=Increment after Writing)
OCPSB_AUTOINC = 7
OBPIF_AUTOINC = OCPSF_AUTOINC
OBPIB_AUTOINC = OCPSB_AUTOINC


; --
; -- OCPD/OBPD ($FF6B)
; -- Object Color Palette Data (aka Object Background Palette Data) (R/W)
; --
rOCPD = $FF6B
rOBPD = rOCPD


; --
; -- OPRI ($FF6C)
; -- Object Priority Mode (R/W)
; -- CGB Only

; --
; -- Priority can be changed only from the boot ROM
; --
rOPRI = $FF6C

OPRI_OAM   = 0 ; Prioritize objects by location in OAM (CGB Mode default)
OPRI_COORD = 1 ; Prioritize objects by x-coordinate (Non-CGB Mode default)



; --
; -- SMBK/SVBK ($FF70)
; -- Select Main RAM Bank (R/W)
; --
; -- Bit 2-0 - Bank Specification (0,1: Specify Bank 1; 2-7: Specify Banks 2-7)
; --
rSVBK = $FF70
rSMBK = rSVBK


; --
; -- PCM12 ($FF76)
; -- Sound channel 1&2 PCM amplitude (R)
; --
; -- Bit 7-4 - Copy of sound channel 2's PCM amplitude
; -- Bit 3-0 - Copy of sound channel 1's PCM amplitude
; --
rPCM12 = $FF76


; --
; -- PCM34 ($FF77)
; -- Sound channel 3&4 PCM amplitude (R)
; --
; -- Bit 7-4 - Copy of sound channel 4's PCM amplitude
; -- Bit 3-0 - Copy of sound channel 3's PCM amplitude
; --
rPCM34 = $FF77


; --
; -- IE ($FFFF)
; -- Interrupt Enable (R/W)
; --
rIE = $FFFF

IEF_HILO   = %00010000 ; Transition from High to Low of Pin number P10-P13
IEF_SERIAL = %00001000 ; Serial I/O transfer end
IEF_TIMER  = %00000100 ; Timer Overflow
IEF_STAT   = %00000010 ; STAT
IEF_VBLANK = %00000001 ; V-Blank

IEB_HILO   = 4
IEB_SERIAL = 3
IEB_TIMER  = 2
IEB_STAT   = 1
IEB_VBLANK = 0


; ***************************************************************************
; *
; * Flags common to multiple sound channels
; *
; ***************************************************************************

; --
; -- Square wave duty cycle
; --
; -- Can be used with AUD1LEN and AUD2LEN
; -- See AUD1LEN for more info
; --
AUDLEN_DUTY_12_5    = %00000000 ; 12.5%
AUDLEN_DUTY_25      = %01000000 ; 25%
AUDLEN_DUTY_50      = %10000000 ; 50%
AUDLEN_DUTY_75      = %11000000 ; 75%


; --
; -- Audio envelope flags
; --
; -- Can be used with AUD1ENV, AUD2ENV, AUD4ENV
; -- See AUD1ENV for more info
; --
AUDENV_UP           = %00001000
AUDENV_DOWN         = %00000000


; --
; -- Audio trigger flags
; --
; -- Can be used with AUD1HIGH, AUD2HIGH, AUD3HIGH
; -- See AUD1HIGH for more info
; --
AUDHIGH_RESTART     = %10000000
AUDHIGH_LENGTH_ON   = %01000000
AUDHIGH_LENGTH_OFF  = %00000000


; ***************************************************************************
; *
; * CPU values on bootup (a=type, b=qualifier)
; *
; ***************************************************************************

BOOTUP_A_DMG    = $01 ; Dot Matrix Game
BOOTUP_A_CGB    = $11 ; Color Game Boy
BOOTUP_A_MGB    = $FF ; Mini Game Boy (Pocket Game Boy)

; if a=BOOTUP_A_CGB, bit 0 in b can be checked to determine if real CGB or
; other system running in GBC mode
BOOTUP_B_CGB    = %00000000
BOOTUP_B_AGB    = %00000001   ; GBA, GBA SP, Game Boy Player, or New GBA SP


; ***************************************************************************
; *
; * Interrupt vector addresses
; *
; ***************************************************************************

INT_HANDLER_VBLANK = $0040
INT_HANDLER_STAT   = $0048
INT_HANDLER_TIMER  = $0050
INT_HANDLER_SERIAL = $0058
INT_HANDLER_JOYPAD = $0060


; ***************************************************************************
; *
; * Header
; *
; ***************************************************************************

; *
; * Nintendo scrolling logo
; * (Code won't work on a real Game Boy)
; * (if next lines are altered.)
NINTENDO_LOGO = 0xCE_ED_66_66_CC_0D_00_0B_03_73_00_83_00_0C_00_0D_00_08_11_1F_88_89_00_0E_DC_CC_6E_E6_DD_DD_D9_99_BB_BB_67_63_6E_0E_EC_CC_DD_DC_99_9F_BB_B9_33_3E

; $0143 Color Game Boy compatibility code
CART_COMPATIBLE_DMG     = $00
CART_COMPATIBLE_DMG_GBC = $80
CART_COMPATIBLE_GBC     = $C0

; $0146 Game Boy/Super Game Boy indicator
CART_INDICATOR_GB       = $00
CART_INDICATOR_SGB      = $03

; $0147 Cartridge type
CART_ROM                     = $00
CART_ROM_MBC1                = $01
CART_ROM_MBC1_RAM            = $02
CART_ROM_MBC1_RAM_BAT        = $03
CART_ROM_MBC2                = $05
CART_ROM_MBC2_BAT            = $06
CART_ROM_RAM                 = $08
CART_ROM_RAM_BAT             = $09
CART_ROM_MMM01               = $0B
CART_ROM_MMM01_RAM           = $0C
CART_ROM_MMM01_RAM_BAT       = $0D
CART_ROM_MBC3_BAT_RTC        = $0F
CART_ROM_MBC3_RAM_BAT_RTC    = $10
CART_ROM_MBC3                = $11
CART_ROM_MBC3_RAM            = $12
CART_ROM_MBC3_RAM_BAT        = $13
CART_ROM_MBC5                = $19
CART_ROM_MBC5_RAM            = $1A
CART_ROM_MBC5_RAM_BAT        = $1B
CART_ROM_MBC5_RUMBLE         = $1C
CART_ROM_MBC5_RAM_RUMBLE     = $1D
CART_ROM_MBC5_RAM_BAT_RUMBLE = $1E
CART_ROM_MBC7_RAM_BAT_GYRO   = $22
CART_ROM_POCKET_CAMERA       = $FC
CART_ROM_BANDAI_TAMA5        = $FD
CART_ROM_HUDSON_HUC3         = $FE
CART_ROM_HUDSON_HUC1         = $FF

; $0148 ROM size
; these are kilobytes
CART_ROM_32KB   = $00 ; 2 banks
CART_ROM_64KB   = $01 ; 4 banks
CART_ROM_128KB  = $02 ; 8 banks
CART_ROM_256KB  = $03 ; 16 banks
CART_ROM_512KB  = $04 ; 32 banks
CART_ROM_1024KB = $05 ; 64 banks
CART_ROM_2048KB = $06 ; 128 banks
CART_ROM_4096KB = $07 ; 256 banks
CART_ROM_8192KB = $08 ; 512 banks
CART_ROM_1152KB = $52 ; 72 banks
CART_ROM_1280KB = $53 ; 80 banks
CART_ROM_1536KB = $54 ; 96 banks

; $0149 SRAM size
; these are kilobytes
CART_SRAM_NONE  = $00
CART_SRAM_8KB   = $02 ; 1 bank
CART_SRAM_32KB  = $03 ; 4 banks
CART_SRAM_128KB = $04 ; 16 banks

; $014A Destination code
CART_DEST_JAPANESE     = $00
CART_DEST_NON_JAPANESE = $01


; ***************************************************************************
; *
; * Keypad related
; *
; ***************************************************************************

PADF_DOWN   = $80
PADF_UP     = $40
PADF_LEFT   = $20
PADF_RIGHT  = $10
PADF_START  = $08
PADF_SELECT = $04
PADF_B      = $02
PADF_A      = $01

PADB_DOWN   = $7
PADB_UP     = $6
PADB_LEFT   = $5
PADB_RIGHT  = $4
PADB_START  = $3
PADB_SELECT = $2
PADB_B      = $1
PADB_A      = $0


; ***************************************************************************
; *
; * Screen related
; *
; ***************************************************************************

SCRN_X    = 160 ; Width of screen in pixels
SCRN_Y    = 144 ; Height of screen in pixels. Also corresponds to the value in LY at the beginning of VBlank.
SCRN_X_B  = 20  ; Width of screen in bytes
SCRN_Y_B  = 18  ; Height of screen in bytes

SCRN_VX   = 256 ; Virtual width of screen in pixels
SCRN_VY   = 256 ; Virtual height of screen in pixels
SCRN_VX_B = 32  ; Virtual width of screen in bytes
SCRN_VY_B = 32  ; Virtual height of screen in bytes


; ***************************************************************************
; *
; * OAM related
; *
; ***************************************************************************

; OAM attributes
; each entry in OAM RAM is 4 bytes (sizeof_OAM_ATTRS)
OAMA_Y              = 0    ; y pos plus 16
OAMA_X              = 1    ; x pos plus 8
OAMA_TILEID         = 2    ; tile id
OAMA_FLAGS          = 3    ; flags (see below)
sizeof_OAM_ATTRS    = 4 

OAM_Y_OFS = 16 ; add this to a screen-relative Y position to get an OAM Y position
OAM_X_OFS = 8  ; add this to a screen-relative X position to get an OAM X position

OAM_COUNT           = 40  ; number of OAM entries in OAM RAM

; flags
OAMF_PRI        = %10000000 ; Priority
OAMF_YFLIP      = %01000000 ; Y flip
OAMF_XFLIP      = %00100000 ; X flip
OAMF_PAL0       = %00000000 ; Palette number; 0,1 (DMG)
OAMF_PAL1       = %00010000 ; Palette number; 0,1 (DMG)
OAMF_BANK0      = %00000000 ; Bank number; 0,1 (GBC)
OAMF_BANK1      = %00001000 ; Bank number; 0,1 (GBC)

OAMF_PALMASK    = %00000111 ; Palette (GBC)

OAMB_PRI        = 7 ; Priority
OAMB_YFLIP      = 6 ; Y flip
OAMB_XFLIP      = 5 ; X flip
OAMB_PAL1       = 4 ; Palette number; 0,1 (DMG)
OAMB_BANK1      = 3 ; Bank number; 0,1 (GBC)