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
test r15 b
test r255b ; error: failed / note:_:10: within / note:_:2: within / error:_:15: out of range
test r255 b ; error: failed / note:_:10: within / note:_:2: within / error:_:16: out of range