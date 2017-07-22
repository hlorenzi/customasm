extern crate customasm;


fn main()
{
	let mut fileserver = customasm::util::FileServerMock::new();
	fileserver.add("test",
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
		
		hllwrld {h}, {w} -> (h + pc * 123)[15:0]
	");
	
	let mut reporter = customasm::diagn::Reporter::new();
	let instrset = customasm::InstrSet::from_src(&mut reporter, &fileserver, "test");
	
	println!("{:#?}", instrset);
	
	reporter.print_all(&fileserver);
}