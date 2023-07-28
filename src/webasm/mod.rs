use crate::*;
use std::mem;
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn wasm_assemble(format: u32, src: *mut String) -> *mut String {
    let src = mem::transmute::<_, &String>(src);

    let mut fileserver = util::FileServerMock::new();
    fileserver.add("asm", src.clone());

    let assemble = |report: diagn::RcReport,
                    fileserver: &util::FileServerMock,
                    filename: &str|
     -> Result<String, ()> {
        let mut asm = asm::Assembler::new();
        asm.register_file(filename);
        let output = asm.assemble(report.clone(), fileserver, 10)?;

        let binary = output.binary;
        match format {
            0 => Ok(binary.format_annotated_hex(fileserver, output.state.cur_wordsize)),
            1 => Ok(binary.format_annotated_bin(fileserver, output.state.cur_wordsize)),

            2 => Ok(binary.format_hexdump()),
            3 => Ok(binary.format_bindump()),
            4 => Ok(binary.format_hexstr()),
            5 => Ok(binary.format_hexline(output.state.cur_wordsize)),
            6 => Ok(binary.format_binstr()),
            7 => Ok(binary.format_binline(output.state.cur_wordsize)),
            8 => Ok(binary.format_coe(output.state.cur_wordsize)),
            9 => Ok(binary.format_mif(4, output.state.cur_wordsize)),
            10 => Ok(binary.format_mif(1, output.state.cur_wordsize)),
            11 => Ok(binary.format_intelhex()),
            12 => Ok(binary.format_comma(10)),
            13 => Ok(binary.format_comma(16)),
            14 => Ok(binary.format_c_array(10)),
            15 => Ok(binary.format_c_array(16)),
            16 => Ok(binary.format_vhdl_b_array(output.state.cur_wordsize)),
            17 => Ok(binary.format_vhdl_h_array(output.state.cur_wordsize)),
            18 => Ok(binary.format_logisim(8)),
            19 => Ok(binary.format_logisim(16)),
            _ => unreachable!(),
        }
    };

    let report = diagn::RcReport::new();

    let output = match assemble(report.clone(), &fileserver, "asm") {
        Ok(output) => output,
        Err(_) => {
            let mut err = Vec::<u8>::new();
            report.print_all(&mut err, &fileserver);
            String::from_utf8(err).unwrap()
        }
    };

    wasm_string_new_with(output)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_get_version() -> *mut String {
    wasm_string_new_with(format!("v{}", env!("CARGO_PKG_VERSION")))
}

#[no_mangle]
pub unsafe extern "C" fn wasm_string_new(len: u32) -> *mut String {
    let mut s = Box::new(String::new());
    for _ in 0..len {
        s.push_str("\0");
    }

    Box::into_raw(s)
}

pub unsafe fn wasm_string_new_with<S>(s: S) -> *mut String
where
    S: Into<String>,
{
    let s = Box::new(s.into());
    Box::into_raw(s)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_string_drop(s: *mut String) {
    let s = Box::from_raw(s);
    drop(s);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_string_get_len(s: *mut String) -> u32 {
    mem::transmute::<_, &mut String>(s).len() as u32
}

#[no_mangle]
pub unsafe extern "C" fn wasm_string_get_byte(s: *mut String, index: u32) -> u8 {
    ptr::read(
        mem::transmute::<_, &mut String>(s)
            .as_ptr()
            .offset(index as isize),
    )
}

#[no_mangle]
pub unsafe extern "C" fn wasm_string_set_byte(s: *mut String, index: u32, value: u8) {
    let bytes = mem::transmute::<_, &mut String>(s).as_ptr();
    ptr::write(
        mem::transmute::<_, *mut u8>(bytes).offset(index as isize),
        value,
    )
}
