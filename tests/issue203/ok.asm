#subruledef foo
{
    add => 0xaa
    sub => 0xbb
}

#ruledef Bar
{
    {f: foo}d => f
}

addd ; = 0xaa
add d ; = 0xaa
subd ; = 0xbb
sub d ; = 0xbb