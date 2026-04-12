#ruledef
{
    echoi {i: u8} => 0xff @ i

    printi {v} => {
        $assert(v >> (8 * 9) == 0)
        $assert(v > 0)
        next = (v >> 8)
        asm {
            printi {next}
            echoi {v} & 0xff
        }
    }

    printi {v} => {
        $assert(v == 0)
        asm {}
    }
}

printi "12345" ; = 0xff31_ff32_ff33_ff34_ff35