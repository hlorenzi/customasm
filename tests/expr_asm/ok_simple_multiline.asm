#ruledef
{
    hlt => 0x12
    stop => asm {
        hlt
        hlt
        hlt
    }
}

stop ; = 0x121212
stop ; = 0x121212
hlt  ; = 0x12