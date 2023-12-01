#[cfg(not(target_arch = "wasm32"))]
use colored::*;

pub enum Color {
    Red,
    Yellow,
    Green,
    BrightGreen,
    Blue,
    BrightBlue,
    Magenta,
    Cyan,
    BrightCyan,
    White,
    Black,
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => ($crate::logging::log("[General]", $crate::logging::Color::BrightBlue, &format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => ($crate::logging::warn(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! error {
    ($($t:tt)*) => ($crate::logging::error(&format_args!($($t)*).to_string()))
}

// Specific log categories

#[macro_export]
macro_rules! log_render {
    ($($t:tt)*) => ($crate::logging::log("[LoiRender]", $crate::logging::Color::BrightCyan, &format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! log_scripting {
    ($($t:tt)*) => ($crate::logging::log("[LoiScripting]", $crate::logging::Color::Cyan, &format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! log_asset {
    ($($t:tt)*) => ($crate::logging::log("[LoiAsset]", $crate::logging::Color::Magenta, &format_args!($($t)*).to_string()))
}

pub fn log(prefix: &str, prefix_color: Color, s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&s.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{} {}", colorize(prefix, prefix_color), s);
    }
}

pub fn warn(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::warn_1(&s.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", s);
    }
}

pub fn error(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::error_1(&s.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", s);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn colorize(s: &str, color: Color) -> String {
    match color {
        Color::Red => s.red().to_string(),
        Color::Yellow => s.yellow().to_string(),
        Color::Green => s.green().to_string(),
        Color::BrightGreen => s.bright_green().to_string(),
        Color::Blue => s.blue().to_string(),
        Color::BrightBlue => s.bright_blue().to_string(),
        Color::Magenta => s.magenta().to_string(),
        Color::Cyan => s.cyan().to_string(),
        Color::BrightCyan => s.bright_cyan().to_string(),
        Color::White => s.white().to_string(),
        Color::Black => s.black().to_string(),
    }
}
