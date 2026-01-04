#subruledef Register {
    r{register: u4} => register
}

#subruledef RegisterSize {
    b => 0xb
}

#ruledef {
    test {register: Register}{size: RegisterSize} => size @ register
}

test r15b
test🤣 r15 b ; error: no match
test r🤣15b  ; error: no match
test r1🤣5 b ; error: no match