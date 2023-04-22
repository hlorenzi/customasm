#ruledef
{
    hlt => 0x12
    stop => asm {
        hlt
        #d 0x34
        hlt
    }
}

stop ; error: failed / note:_:4: within / error:_:6: invalid
stop
hlt