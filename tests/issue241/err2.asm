#ruledef {
	abc {n: u8}	=> n
}
test:
.A = .B ; error: converge
.B = .A ; error: converge
	abc .A ; error: converge