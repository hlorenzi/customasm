#ruledef
{
    echoi {i: u8} => 0xff @ i @ ADDRMAP.OUTPUT`8

    printi {v} => {
        assert(v >> (8 * 9) == 0)
        assert(v > 0)
        next = (v >> 8)
        asm {
            printi {next}
            echoi {v} & 0xff
        }
    }

    printi {v} => {
        assert(v == 0)
        asm {}
    }
}

printi "12345" ; = 0xff3144_ff3244_ff3344_ff3444_ff3544

#addr 0x40
ADDRMAP:
    .OUTPUT = ADDRMAP + 0x4