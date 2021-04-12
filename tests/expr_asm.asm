; :::
; simple
#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit x }
}

emit 0x12     ; = 0x12
test 0x12     ; = 0x12
test 0x10 + 2 ; = 0x12


; :::
; concat
#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit x } @ asm { emit 0xff }
}

emit 0x12     ; = 0x12
test 0x12     ; = 0x12ff
test 0x10 + 2 ; = 0x12ff


; :::
; multi-line
#ruledef
{
    emit {x: i8} => 0x11 @ x
    load {x: i8} => 0x22 @ x
    test {x} => asm
    {
        emit x
        emit 0xff
        load x
    }
}

test 0x12 ; = 0x1112_11ff_2212


; :::
; inner expression
#ruledef
{
    emit {x: i8} => x
    test {x} => asm { emit x * 9 }
}

test 2     ; = 0x12
test 1 + 1 ; = 0x12


; :::
; multiple captured variables
#ruledef
{
    emit {x: i8}, {y: i8} => x @ y
    test {x}, {y} => asm { emit x, y }
}

test 0x12, 0x34         ; = 0x1234
test 0x10 + 2, 0x30 + 4 ; = 0x1234


; :::
; captured local variable
#ruledef
{
    emit {x: i8} => x
    test {x} =>
    {
        y = 0x10
        asm { emit x + y }
    }
}

test 2 ; = 0x12


; :::
; from different ruledef blocks
#ruledef
{
    test {x} => asm { emit x }
}

#ruledef
{
    emit {x: i8} => x
}

emit 0x12 ; = 0x12
test 0x12 ; = 0x12


; :::
; assert resolution
#ruledef
{
    emit {x: i8} =>
    {
        assert(x < 0x10)
        0x11 @ x
    }
    emit {x: i8} =>
    {
        assert(x >= 0x10)
        0x22 @ x
    }
    test {x} => asm { emit x }
}

emit 0x08 ; = 0x1108
emit 0x12 ; = 0x2212
test 0x08 ; = 0x1108
test 0x12 ; = 0x2212


; :::
; no match
#ruledef
{
    test {x} => asm { unknown x }
}

test 0x12 ; error: failed / error: :4: no match


; :::
; multiple matches
#ruledef
{
    emit {x: i8} => x
    emit {x: i8} => x
    test {x} => asm { emit x }
}

test 0x12 ; error: failed / error: :6: multiple


; :::
; inner error
#ruledef
{
    emit {x} => x / 0
    test {x} => asm { emit x }
}

test 12 ; error: failed / error: :4: zero


; :::
; inner error
#ruledef
{
    emit {x} => x
    test {x} => asm { emit x }
}

test 12 ; error: failed / error: :5: infer


; :::
; from forward-referenced ruledef
#ruledef
{
    test {x} => asm { emit x }
}

; weird error?!
test 0x12 ; error: converge

#ruledef
{
    emit {x: i8} => x
}