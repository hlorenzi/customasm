#include "<std>/platform/gameboy/constants.asm"
CART_TYPE = CART_ROM

#include "<std>/platform/gameboy/common_banks.asm"

#bankdef rom {
    #addr 0x0000
    #size 0x8000
    #outp 0x0000
}

#bank rom