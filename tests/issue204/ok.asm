#ruledef foo
{
    f{e:u4}	=> e
    f{e:u4}+{m:foo}	=> e @ m
}

#ruledef bar
{
    |{f:foo}| => 0xa @ f
    ){f:foo}( => 0xb @ f
    ]{f:foo}[ => 0xc @ f
    ({f:foo}) => 0xd @ f
    [{f:foo}] => 0xe @ f
}

f1+f2+f3+f4   ; = 0x1234
|f1+f2+f3+f4| ; = 0xa1234
)f1+f2+f3+f4( ; = 0xb1234
]f1+f2+f3+f4[ ; = 0xc1234
(f1+f2+f3+f4) ; = 0xd1234
[f1+f2+f3+f4] ; = 0xe1234