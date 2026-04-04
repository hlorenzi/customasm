#ruledef {
	abc {n: u8}	=> n
}
test:
.A = .A ; error: converge
	abc .A ; error: converge