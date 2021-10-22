#ruledef {
    emit {x: u8} => x 

    run => asm 
    {
        test
        emit 0x10 
    }

    test => asm
    {
        test2 end
        test2 end 
        end:
    }

    test2 {l: u32} => asm
    {
        emit l
        emit end
        end:
    }
}

run ; = 0x04_02_04_04_10 
emit $ ; = 0x05