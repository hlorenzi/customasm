#ruledef mode
{
    eq => 0xff
}

#ruledef
{
    b{m: mode} => m
    j{m: mode} => asm { b{m} }
}

jeq ; = 0xff