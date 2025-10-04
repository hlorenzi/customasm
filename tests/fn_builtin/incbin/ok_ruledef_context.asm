#include "inner/rule.asm"

#d incbin("data1.bin") ; = 0x68656c6c6f
hlt ; = 0x676f6f64627965
ld incbin("data1.bin") ; = 0x68656c6c6f
st "data1.bin" ; = 0x676f6f64627965