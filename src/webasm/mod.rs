use asm::AssemblerState;
use diagn::RcReport;
use util::FileServerMock;
use std::mem;
use std::ptr;


#[no_mangle]
pub unsafe extern fn wasm_assemble(format: u32, src: *mut String) -> *mut String
{
	let src = mem::transmute::<_, &String>(src);
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("asm", src.clone());
	
	let assemble = |report: RcReport, fileserver: &FileServerMock, filename: &str| -> Result<String, ()>
	{
		let mut asm = AssemblerState::new();
		asm.process_file(report.clone(), fileserver, filename)?;
		asm.wrapup(report.clone())?;
		
		let output = asm.get_binary_output();
		match format
		{
			0 => Ok(output.generate_hexdump(0, output.len())),
			1 => Ok(output.generate_bindump(0, output.len())),
			2 => Ok(output.generate_hexstr(0, output.len())),
			3 => Ok(output.generate_binstr(0, output.len())),
			_ => unreachable!()
		}
	};
		
	let report = RcReport::new();
	
	let output = match assemble(report.clone(), &fileserver, "asm")
	{
		Ok(output) => output,
		Err(_) =>
		{
			let mut err = Vec::<u8>::new();
			report.print_all(&mut err, &fileserver);
			String::from_utf8(err).unwrap()
		}
	};
	
	wasm_string_new_with(output)
}


#[no_mangle]
pub unsafe extern fn wasm_string_new(len: u32) -> *mut String
{
	let mut s = Box::new(String::new());
	for _ in 0..len
		{ s.push_str("\0"); }
	
	Box::into_raw(s)
}


pub unsafe fn wasm_string_new_with<S>(s: S) -> *mut String
where S: Into<String>
{
	let s = Box::new(s.into());
	Box::into_raw(s)
}


#[no_mangle]
pub unsafe extern fn wasm_string_drop(s: *mut String)
{
	let s = Box::from_raw(s);
	drop(s);
}


#[no_mangle]
pub unsafe extern fn wasm_string_get_len(s: *mut String) -> u32
{
	mem::transmute::<_, &mut String>(s).len() as u32
}


#[no_mangle]
pub unsafe extern fn wasm_string_get_byte(s: *mut String, index: u32) -> u8
{
	ptr::read(mem::transmute::<_, &mut String>(s).as_ptr().offset(index as isize))
}


#[no_mangle]
pub unsafe extern fn wasm_string_set_byte(s: *mut String, index: u32, value: u8)
{
	let bytes = mem::transmute::<_, &mut String>(s).as_ptr();
	ptr::write(mem::transmute::<_, *mut u8>(bytes).offset(index as isize), value)
}