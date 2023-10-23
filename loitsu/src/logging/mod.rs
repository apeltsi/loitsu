#[macro_export]
macro_rules! log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::logging::log(&format_args!($($t)*).to_string()))
}

pub fn log(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&s.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{}", s);
    }
}
