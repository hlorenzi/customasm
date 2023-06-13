#d 0xab ; = 0xab

#if true
{
    #d 0xcd ; = 0xcd

    #if false
    {
        #d 0x88
        
        #if true
        {
            #d 0x77
        }
    }
    
    #if true
    {
        #d 0x66 ; = 0x66
        
        #if false
        {
            #d 0x55
        }
        
        #if true
        {
            #d 0x44 ; = 0x44
        }
    }
}

#d 0xef ; = 0xef