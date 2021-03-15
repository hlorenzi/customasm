; :::
#subruledef REG {
    my_reg => 0xA
}
#ruledef {
    test {Addr: u16} => 0x1 @ Addr
    test {register: REG} + {Imm: u16} => register`4 @ Imm
}

test 1 ; = 0x10001
test 1 + 1 ; = 0x10002
test my_reg + 1 ; = 0xa0001

; :::
#subruledef REG {
    my_reg => 0xA
}
#ruledef {
    test {Addr: u16} => 0x1 @ Addr
    test {register: REG} + {Imm: u16} => register`4 @ Imm_Unknown
}

test my_reg + 1
; error: :9: failed to resolve
; error: :6: unknown