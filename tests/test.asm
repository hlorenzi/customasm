#rulesdef cpu6502
{
    halt => 0xab
    nop => 0xcd
    ld.x => 0xef
}

#use cpu6502

halt
nop
ld.x

err
err 123 err

#use err