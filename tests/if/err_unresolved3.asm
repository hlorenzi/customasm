#d 0xab

x = 0

#if x == y ; error: unresolved condition / error: unknown symbol `y`
{
    y = 0

    #d 0xcd
}

#d 0xef