use crate::*;


#[no_mangle]
pub unsafe extern fn wasm_assemble(
	format: u32,
	src: *mut String)
	-> *mut String
{
	let virtual_filename = "asm";

	let src = std::mem::transmute::<_, &String>(src);
			
	let mut report = diagn::Report::new();

	let mut fileserver = util::FileServerMock::new();
	fileserver.add(virtual_filename, src.clone());

	let opts = asm::AssemblyOptions::new();

	let assembly = asm::assemble(
		&mut report,
		&opts,
		&mut fileserver,
		&[virtual_filename]);
	
	let output = {
		match assembly.output
		{
			Some(o) => o,
			None =>
			{
				let mut err = Vec::<u8>::new();
				report.print_all(&mut err, &fileserver);
				return wasm_string_new_with(
					String::from_utf8(err).unwrap());
			}
		}
	};
	
	let formatted = {
		match format
		{
			0 => output.format_annotated_hex(&fileserver),
			1 => output.format_annotated_bin(&fileserver),
			2 => output.format_hexdump(),
			3 => output.format_bindump(),
			4 => output.format_hexstr(),
			5 => output.format_binstr(),
			6 => output.format_mif(),
			7 => output.format_intelhex(),
			8 => output.format_separator(10, ", "),
			9 => output.format_separator(16, ", "),
			10 => output.format_separator(10, " "),
			11 => output.format_separator(16, " "),
			12 => output.format_c_array(10),
			13 => output.format_c_array(16),
			14 => output.format_logisim(8),
			15 => output.format_logisim(16),
			_ => unreachable!()
		}
	};

	wasm_string_new_with(formatted)
}


#[no_mangle]
pub unsafe extern fn wasm_get_version() -> *mut String
{
	wasm_string_new_with(format!("v{}", env!("CARGO_PKG_VERSION")))
}


#[no_mangle]
pub unsafe extern fn wasm_string_new(len: u32) -> *mut String
{
	let mut s = Box::new(String::new());
	for _ in 0..len
	{
		s.push_str("\0");
	}
	
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
	std::mem::transmute::<_, &mut String>(s).len() as u32
}


#[no_mangle]
pub unsafe extern fn wasm_string_get_byte(s: *mut String, index: u32) -> u8
{
	std::ptr::read(
		std::mem::transmute::<_, &mut String>(s)
			.as_ptr()
			.offset(index as isize))
}


#[no_mangle]
pub unsafe extern fn wasm_string_set_byte(s: *mut String, index: u32, value: u8)
{
	let bytes = std::mem::transmute::<_, &mut String>(s).as_ptr();
	std::ptr::write(
		std::mem::transmute::<_, *mut u8>(bytes)
			.offset(index as isize),
		value)
}