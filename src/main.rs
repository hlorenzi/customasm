extern crate customasm;


fn main()
{
	let mut fileserver = customasm::FileServerMock::new();
	fileserver.add("instrset",
	"
		#align 8
		
		halt         ->  8'0x00 ; stop the machine
		jmp {addr!}  -> 16'0x10
		
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
		halt
		jmp 0xab
		store 0xcd
		load 0xef + 0xfe * (0xcc - 0xbb)
		load 0x12, 0x34
		load 0x33, 0x44, 0x55
	");
	
	let mut reporter = customasm::Reporter::new();
	
	if let Some(instrset) = customasm::read_instrset(&mut reporter, &fileserver, "instrset")
	{
		//println!("{:#?}", instrset);
		
		if let Some(assembled) = customasm::assemble(&mut reporter, &instrset, &fileserver, "code")
		{
			println!("{:#?}", assembled);
		}
	}
	
	reporter.print_all(&fileserver);
}