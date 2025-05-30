//! Macros for multi-level formatted logging used by
//! [ArceOS](https://github.com/arceos-org/arceos).
//!
//! The log macros, in descending order of level, are: [`error!`], [`warn!`],
//! [`info!`], [`debug!`], and [`trace!`].
//!
//! If it is used in `no_std` environment, the users need to implement the
//! [`LogIf`] to provide external functions such as console output.
//!
//! To use in the `std` environment, please enable the `std` feature:
//!
//! ```toml
//! [dependencies]
//! axlog = { version = "0.1", features = ["std"] }
//! ```
//!
//! # Cargo features:
//!
//! - `std`: Use in the `std` environment. If it is enabled, you can use console
//!   output without implementing the [`LogIf`] trait. This is disabled by default.
//! - `log-level-off`: Disable all logging. If it is enabled, all log macros
//!   (e.g. [`info!`]) will be optimized out to a no-op in compilation time.
//! - `log-level-error`: Set the maximum log level to `error`. Any macro
//!   with a level lower than [`error!`] (e.g, [`warn!`], [`info!`], ...) will be
//!   optimized out to a no-op.
//! - `log-level-warn`, `log-level-info`, `log-level-debug`, `log-level-trace`:
//!   Similar to `log-level-error`.
//!
//! # Examples
//!
//! ```
//! use axlog::{debug, error, info, trace, warn};
//!
//! // Initialize the logger.
//! axlog::init();
//! // Set the maximum log level to `info`.
//! axlog::set_max_level("info");
//!
//! // The following logs will be printed.
//! error!("error");
//! warn!("warn");
//! info!("info");
//!
//! // The following logs will not be printed.
//! debug!("debug");
//! trace!("trace");
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate log;

use core::fmt::{self, Write};
use core::str::FromStr;

use log::{Level, LevelFilter, Log, Metadata, Record};

#[cfg(not(feature = "std"))]
use crate_interface::call_interface;

pub use log::{debug, error, info, trace, warn};

/// Prints to the console.
///
/// Equivalent to the [`ax_println!`] macro except that a newline is not printed at
/// the end of the message.
#[macro_export]
macro_rules! ax_print {
    ($($arg:tt)*) => {
        $crate::__print_impl(format_args!($($arg)*));
    }
}

/// Prints to the console, with a newline.
#[macro_export]
macro_rules! ax_println {
    () => { $crate::ax_print!("\n") };
    ($($arg:tt)*) => {
        $crate::__print_impl(format_args!("{}\n", format_args!($($arg)*)));
    }
}

macro_rules! with_color {
    ($color_code:expr, $($arg:tt)*) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[m", $color_code as u8, format_args!($($arg)*))
    }};
}

#[repr(u8)]
#[allow(dead_code)]
enum ColorCode {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

/// Extern interfaces that must be implemented in other crates.
#[crate_interface::def_interface]
pub trait LogIf {
    /// Writes a string to the console.
    fn console_write_str(s: &str);

    /// Gets current clock time.
    fn current_time() -> core::time::Duration;

    /// Gets current CPU ID.
    ///
    /// Returns [`None`] if you don't want to show the CPU ID in the log.
    fn current_cpu_id() -> Option<usize>;

    /// Gets current task ID.
    ///
    /// Returns [`None`] if you don't want to show the task ID in the log.
    fn current_task_id() -> Option<u64>;
}

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                std::print!("{}", s);
            } else {
                call_interface!(LogIf::console_write_str, s);
            }
        }
        Ok(())
    }
}

impl Log for Logger {
    #[inline]
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let line = record.line().unwrap_or(0);
        let path = record.target();
        let args_color = match level {
            Level::Error => ColorCode::Red,
            Level::Warn => ColorCode::Yellow,
            Level::Info => ColorCode::Green,
            Level::Debug => ColorCode::Cyan,
            Level::Trace => ColorCode::BrightBlack,
        };

        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                __print_impl(with_color!(
                    ColorCode::White,
                    "[{time} {path}:{line}] {args}\n",
                    time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                    path = path,
                    line = line,
                    args = with_color!(args_color, "{}", record.args()),
                ));
            } else {
                let cpu_id = call_interface!(LogIf::current_cpu_id);
                let tid = call_interface!(LogIf::current_task_id);
                let now = call_interface!(LogIf::current_time);
                if let Some(cpu_id) = cpu_id {
                    if let Some(tid) = tid {
                        // show CPU ID and task ID
                        __print_impl(with_color!(
                            ColorCode::White,
                            "[{:>3}.{:06} {cpu_id}:{tid} {path}:{line}] {args}\n",
                            now.as_secs(),
                            now.subsec_micros(),
                            cpu_id = cpu_id,
                            tid = tid,
                            path = path,
                            line = line,
                            args = with_color!(args_color, "{}", record.args()),
                        ));
                    } else {
                        // show CPU ID only
                        __print_impl(with_color!(
                            ColorCode::White,
                            "[{:>3}.{:06} {cpu_id} {path}:{line}] {args}\n",
                            now.as_secs(),
                            now.subsec_micros(),
                            cpu_id = cpu_id,
                            path = path,
                            line = line,
                            args = with_color!(args_color, "{}", record.args()),
                        ));
                    }
                } else {
                    // neither CPU ID nor task ID is shown
                    __print_impl(with_color!(
                        ColorCode::White,
                        "[{:>3}.{:06} {path}:{line}] {args}\n",
                        now.as_secs(),
                        now.subsec_micros(),
                        path = path,
                        line = line,
                        args = with_color!(args_color, "{}", record.args()),
                    ));
                }
            }
        }
    }

    fn flush(&self) {}
}

/// Prints the formatted string to the console.
pub fn print_fmt(args: fmt::Arguments) -> fmt::Result {
    use kspin::SpinNoIrq; // TODO: more efficient
    static LOCK: SpinNoIrq<()> = SpinNoIrq::new(());

    let _guard = LOCK.lock();
    
    // heke - color2
    struct RainbowWriter;
    impl fmt::Write for RainbowWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for (i, c) in s.chars().enumerate() {
                let hue = (i as f32 * 5.0) % 360.0;
                let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                Logger.write_fmt(format_args!("\x1b[38;2;{};{};{}m{}", r, g, b, c))?;
            }
            Logger.write_str("\x1b[0m")?;
            Ok(())
        }
    }

    let mut writer = RainbowWriter;
    writer.write_fmt(args)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_div = h / 60.0;
    let x = c * (1.0 - (if h_div % 2.0 > 1.0 { h_div % 2.0 - 1.0 } else { 1.0 - h_div % 2.0 }));
    let m = v - c;
    let (r, g, b) = match h_div as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

#[doc(hidden)]
pub fn __print_impl(args: fmt::Arguments) {
    print_fmt(args).unwrap();
}

/// Initializes the logger.
///
/// This function should be called before any log macros are used, otherwise
/// nothing will be printed.
pub fn init() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(LevelFilter::Warn);
}

/// Set the maximum log level.
///
/// Unlike the features such as `log-level-error`, setting the logging level in
/// this way incurs runtime overhead. In addition, this function is no effect
/// when those features are enabled.
///
/// `level` should be one of `off`, `error`, `warn`, `info`, `debug`, `trace`.
pub fn set_max_level(level: &str) {
    let lf = LevelFilter::from_str(level)
        .ok()
        .unwrap_or(LevelFilter::Off);
    log::set_max_level(lf);
}
