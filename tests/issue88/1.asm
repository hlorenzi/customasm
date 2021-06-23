#subruledef reg {
    ax => 0x0
    bx => 0x1
    cx => 0x2
}

#subruledef memaccess {
    {o: s8}(%{r: reg}) => 0x00 @ r @ o
}

#subruledef arg {
    {m: memaccess} => 0x00 @ m
    %{r: reg} => 0x01 @ r
}

#ruledef {
    mov {a: arg}, {a2: arg} => 0x10 @ a @ a2
}

mov (%ax), %bx ; error: no match