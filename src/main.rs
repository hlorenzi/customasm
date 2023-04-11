extern crate customasmlib;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut fileserver = customasmlib::util::FileServerReal::new();

    if let Err(()) = customasmlib::driver::drive(&args, &mut fileserver) {
        std::process::exit(1);
    }
}
