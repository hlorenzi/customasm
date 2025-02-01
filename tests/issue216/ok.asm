#subruledef acc {
    a => 0xaa
    b => 0xbb
}

#subruledef paropts {
    {} => 0x00
    FOO => 0x01
}

#ruledef {
    abs {reg: acc} {p: paropts} => 0xee @ reg @ p
}

abs a FOO ; = 0xeeaa01
abs a ; = 0xeeaa00