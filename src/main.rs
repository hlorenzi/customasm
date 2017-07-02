extern crate customasm;


fn main()
{
	let src =
	"
		#align 8
		
		halt         -> 8'0x00 ; stop the machine
		jmp {addr}   -> 8'0x10 addr[15:0]
		load {addr}  -> 8'0x20 addr[15:0]
		store {addr} -> 8'0x30 addr[15:0]
	";
	
	let chars = src.chars().collect::<Vec<_>>();
	
	let tokens = customasm::syntax::tokenize("test", &chars);
	println!("{:#?}", tokens);
}