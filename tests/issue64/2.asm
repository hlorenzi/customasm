#subruledef REG {
    my_reg => 0xA
}
#ruledef {
    test {Addr: u16} => 0x1 @ Addr
    test {register: REG} + {Imm: u16} => register`4 @ Imm_Unknown
}

test my_reg + 1 ; error: failed / note:_:6: within / error:_:6: unknown