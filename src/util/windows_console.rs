// From: https://github.com/ogham/rust-ansi-term/blob/master/src/windows.rs


#[cfg(not(windows))]
pub fn enable_windows_ansi_support()
{

}


#[cfg(windows)]
pub fn enable_windows_ansi_support()
{
    #[link(name = "kernel32")]
    unsafe extern "C"
	{
        unsafe fn GetStdHandle(handle: u64) -> *const i32;
        unsafe fn SetConsoleMode(handle: *const i32, mode: u32) -> bool;
    }
	
    unsafe
		{ SetConsoleMode(GetStdHandle(-11i32 as u64), 7); }
}