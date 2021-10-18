#ruledef {
    emit {x:u8} => x

    nested_test => asm
    {
        label:
        emit $
        label2:
        emit $
        emit label 
        emit label2 
        emit $
        test
    }

    test => asm
    {
        label:
        emit $
        label2:
        emit $
        emit label 
        emit label2 
        emit $
    }
}

test ; = 0x00_01_00_01_04
nested_test ; = 0x05_06_05_06_09_0a_0b_0a_0b_0e