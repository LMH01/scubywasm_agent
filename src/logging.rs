#[link(wasm_import_module = "debug")]
unsafe extern "C" {
    fn debug_log(ptr: u32, len: u32);
}

pub fn log_str(message: &str) {
    let ptr = message.as_ptr() as u32;
    let len = message.len() as u32;
    unsafe {
        debug_log(ptr, len);
    };
    print!("a")
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        $crate::logging::log_str(format!($($arg)*).as_ref());
    }};
}