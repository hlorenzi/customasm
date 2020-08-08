--- include

#rulesdef simple
{
    halt => 0x55
}

#use simple

---

halt ; = 0x55

---

halt ; = 0x55
halt ; = 0x55

---

halt
halt
; = 0x5555

---

unk ; error: no match

---

halt
unk ; error: no match
halt