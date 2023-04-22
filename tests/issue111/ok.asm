#ruledef 
{
    sub.neg {a: u8} {b: u8} {r: u8} {j: u8} => a @ b @ r @ j

    jmp {j: u8} => asm { sub.neg Z Z+1 T {j} }
}

jmp main ; = 0x08090410_00000000_00000000_00000001

T: 
#d32 0

Z:
#d32 0
#d32 1

main: