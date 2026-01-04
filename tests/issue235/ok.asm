#subruledef Register {
    r{register: u4} => register
}

#subruledef RegisterSize {
    b => 0xb
}

#ruledef {
    test {register: Register}{size: RegisterSize} => size @ register
}

test r0b ; = 0xb0
test r0 b ; = 0xb0
test r1b ; = 0xb1
test r1 b ; = 0xb1
test r15b ; = 0xbf
test r15 b ; = 0xbf