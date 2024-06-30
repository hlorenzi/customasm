#subruledef foo
{
    f{i:u4} => i
}


#ruledef bar
{
    [{f:foo}] => 0xa @ f
    [{i:u4}]  => 0xb @ i
}


foo:
bar:
    [f10]
    [bar] 
    [foo] ; error: failed / note:_:9: within / note:_:3: within / error: unknown symbol `oo`