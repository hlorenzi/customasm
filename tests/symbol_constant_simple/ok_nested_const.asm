#ruledef test
{
    ld {x}, {y}, {z} => 0x55 @ x`8 @ y`8 @ z`8
}


#const x = 0x11
#const .y = 0x22
#const ..z = 0x33
ld x, .y, ..z ; = 0x55112233
ld x, x.y, x.y.z ; = 0x55112233