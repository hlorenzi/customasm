; ::: include
#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm { emit {r} }
}

; :::

emit a ; = 0xaa
emit b ; = 0xbb
test a ; = 0xaa
test b ; = 0xbb

; :::

test c ; error: no match

; :::

test 0x12 ; error: no match


; ::: include
; :::
#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm
    {
        emit {r}
        emit b
        emit {r}
    }
}

test a ; = 0xaabbaa


; ::: include
; :::
#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm { emit {unknown} }
}

test a ; error: failed / error: :10: unknown


; ::: include
; :::
#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef
{
    emit {r: reg} => r`8
    test {r: reg} => asm { emit r }
}

test a ; error: failed / error: :10: no match


; ::: include
; :::
#ruledef reg
{
    a => 0xaa
    b => 0xbb
}

#ruledef reg2
{
    a => 0xaa
    b => 0xbb
    c => 0xcc
}

#ruledef
{
    emit {r: reg}  => r`8
    test {r: reg2} => asm { emit {r} }
}

test c ; error: failed / error: :17: no match