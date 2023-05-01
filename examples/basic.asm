#ruledef
{
    load r1, {value: i8} => 0x11 @ value
    load r2, {value: i8} => 0x12 @ value
    load r3, {value: i8} => 0x13 @ value
    add  r1, r2          => 0x21
    sub  r3, {value: i8} => 0x33 @ value
    jnz  {address: u16}  => 0x40 @ address
    ret                  => 0x50
}


multiply3x4:
    load r1, 0
    load r2, 3
    load r3, 4
    
    .loop:
        add r1, r2
        sub r3, 1
        jnz .loop
    
    ret