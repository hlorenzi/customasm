#ruledef test
{
    halt => 0x55
}

#macro halt3 => {
    halt
    halt
    halt
}

halt3 ; = 0x55_55_55