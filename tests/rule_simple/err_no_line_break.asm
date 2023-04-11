#ruledef test
{
    halt => 0x55
    nop => 0xaa
}

halt
nop
halt nop ; error: no match
halt
nop