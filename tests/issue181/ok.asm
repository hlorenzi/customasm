#subruledef SELECT
{
	C => 0xc
}

#ruledef
{
	A{sel:SELECT}    => 0x1 @ sel
	AB{sel:SELECT}   => 0x2 @ sel
	ABB{sel:SELECT}	 => 0x3 @ sel
	ABBB{sel:SELECT} => 0x4 @ sel
}

AC    ; = 0x1c
ABC   ; = 0x2c
ABBC  ; = 0x3c
ABBBC ; = 0x4c