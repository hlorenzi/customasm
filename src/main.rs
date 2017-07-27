extern crate customasm;


fn main()
{
	let mut fileserver = customasm::FileServerMock::new();
	fileserver.add("instrset",
	"
		#align 8
		
		halt         ->  8'0x00 ; stop the machine
		jmp {addr}  -> 16'0x10
		
		store {addr} -> 16'0x30, addr[15:0]
		
		load {addr}
			:: addr % 2 == 0, \"addr is not even\"
			:: addr % 3 == 0, \"addr is not multiple of 3\"
			:: addr > 0x10 ; no description
			-> 16'0xffff, addr[7:0]
		
		load {h}, {w} -> (h + pc * 123)[15:0]
	");
	
	fileserver.add("code",
	"
		#addr 16 + 16 + 16
		#write 24
		#res 255
	
		halt
		jmp 0x01
		jmp 0x01
		store 0x02 + abc
		jmp 0x01
		jmp 0x01
		load 0x0
		load 0x09 + 0x01 * (0x17 - 0x10)
		load 0x03, 0x04
		load 0x05, 0x06, 0x07
		load 0x03, 0x04
		load 0x05, 0x06, 0x07
		load 0x03, 0x04
		load 0x05, 0x06, 0x07
	");
	
	let mut report = customasm::Report::new();
	
	if let Ok(instrset) = customasm::read_instrset(&mut report, &fileserver, "instrset")
	{
		//println!("{:#?}", instrset);
		
		if let Ok(assembled) = customasm::assemble(&mut report, &instrset, &fileserver, "code")
		{
			//println!("{:#?}", assembled);
		}
	}
	
	report.print_all(&fileserver);
}