const default_text = window.default_text_override ?? `#ruledef
{
    load r1, {value} => 0x11 @ value\`8
    load r2, {value} => 0x12 @ value\`8
    load r3, {value} => 0x13 @ value\`8
    add  r1, r2      => 0x21
    sub  r3, {value} => 0x33 @ value\`8
    jnz  {address}   => 0x40 @ address\`16
    ret              => 0x50
}


multiply3x4:
    load r1, 0
    load r2, 3
    load r3, 4
    
    .loop:
        add r1, r2
        sub r3, 1
        jnz .loop
    
    ret`;
