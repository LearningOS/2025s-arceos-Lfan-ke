#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("\x1b[1m\x1b[38;2;150;50;255m{}\x1b[0m\r\x1b[K", "[WithColor]: Hello, Arceos!");
}
