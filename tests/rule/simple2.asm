---

#rulesdef simple
{
    halt => 0x55
    nop => 0xaa
}

#use simple

halt ; = 0x55
nop  ; = 0xaa
halt ; = 0x55
nop  ; = 0xaa

---

#rulesdef simple
{
    test#(x$) => 0x55
}

#use simple

test # ( x $ ) ; = 0x55