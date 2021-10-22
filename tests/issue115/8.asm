#ruledef
{
    emit {x: u8} => x

    test => asm
    {
        emit ret
        emit globalLabel
        ret:
    }
}

test ; error: failed / error:_:5: converge
globalLabel: