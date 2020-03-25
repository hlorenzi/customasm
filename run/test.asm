#cpudef {
    #bits 8

    #tokendef lots {
        a = 0
        b = 1
        c = 2
        d = 3
        e = 4
        f = 5
        g = 6
        h = 7
        i = 8
        j = 9
        k = 10
        l = 11
        m = 12
        n = 13
        o = 14
        p = 15
        q = 16
        r = 17
        s = 18
        t = 19
        u = 20
        v = 21
        w = 22
        x = 23
        y = 24
        z = 25
    }

    ;slow -> 8'0
    ;slow {arg0: lots} -> 8'0
    ;slow {arg0: lots} {arg1: lots} -> 8'0
    ;slow {arg0: lots} {arg1: lots} {arg2: lots} -> 8'0
    ;slow {arg0: lots} {arg1: lots} {arg2: lots} {arg3: lots} -> 8'0

    thing with constants -> 0x02
    thing with more constants -> 0x03
    thing with {expr} -> 0x0F @ expr[7:0] @ 0xF0
}

;slow 
;slow a 
;slow a b
;slow a b c
;slow a b c d

thing with 0b00111100
thing with constants
thing with more constants