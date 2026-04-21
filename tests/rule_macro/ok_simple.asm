#ruledef test
{
    halt => 0x55
    halt3 => macro {
        halt
        halt
        halt
    }
}

halt ; = 0x55
halt3 ; = 0x55_55_55