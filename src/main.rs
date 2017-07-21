extern crate customasm;


fn main()
{
	let mut fileserver = customasm::util::FileServerMock::new();
	fileserver.add("test",
	"
		#align 8
		
		halt         -> ; 8'0x00 ; stop the machine
		jmp {addr!}  -> ; 8'0x10 addr[15:0]
		load {addr}  -> ; 8'0x20 addr[15:0]
		store {addr} -> ; 8'0x30 addr[15:0]
		err {a}, {a} -> ;
	");
	
	let mut reporter = customasm::diagn::Reporter::new();
	let instrset = customasm::InstrSet::from_src(&mut reporter, &fileserver, "test");
	
	println!("{:#?}", instrset);
	
	reporter.print_all(&fileserver);
}