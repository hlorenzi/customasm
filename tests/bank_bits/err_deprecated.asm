#bits 3 ; error: deprecated

#ruledef
{
    ld {x: u3} => 0b111 @ x
}

ld 0
label:
ld 5