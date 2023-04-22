#subruledef sub
{
    {a: u8} => a
}

#ruledef
{
    test {a: sub} ({b: u8}) => 0x11 @ a @ b
    test {a: sub}  {b: u8}) => 0x22 @ a @ b

    test2 {a: u8}({b: u8}) => 0x33 @ a @ b
}

test 0 (0) ; = 0x110000
test 0  0) ; = 0x220000

test (0) (0) ; = 0x110000
test (ascii("\0")) (0) ; = 0x110000

x = 0xee
test x (0) ; = 0x11ee00
test (x) (0) ; = 0x11ee00


test2 1(2) ; = 0x330102
test2 (1)(2) ; = 0x330102
test2 (ascii("a"))(2) ; = 0x336102
test2 x(2) ; = 0x33ee02
test2 (x)(2) ; = 0x33ee02