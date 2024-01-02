use std::fmt::Display;

pub trait Numeric: Into<u64> + Display + Copy {}
impl Numeric for u32 {}
impl Numeric for u64 {}

/// Format kilobytes as kiB, MiB or GiB, with 2-digit precision
pub fn format_kb<T: Numeric>(kilobytes: T) -> String {
    let kilobytes = kilobytes.into();
    if kilobytes < 1024 {
        format!("{kilobytes} KiB")
    } else if kilobytes < 1024 * 1024 {
        format!("{:.2} MiB", kilobytes as f32 / 1024f32)
    } else {
        format!("{:.2} GiB", kilobytes as f32 / 1024f32 / 1024f32)
    }
}

pub fn format_bytes<T: Numeric>(bytes: T) -> String {
    let bytes = bytes.into();
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KiB", bytes as f32 / 1024f32)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MiB", bytes as f32 / 1024f32 / 1024f32)
    } else {
        format!("{:.2} GiB", bytes as f32 / 1024f32 / 1024f32 / 1024f32)
    }
}

pub fn format_duration(duration: u64) -> String {
    let seconds = duration % 60;
    let minutes = (duration / 60) % 60;
    let hours = (duration / 3600) % 24;
    if duration < 60 {
        format!("{seconds}s")
    } else if duration < 3600 {
        format!("{minutes}m{seconds}s")
    } else {
        format!("{hours}h{minutes}m{seconds}s")
    }
}

pub trait BytesFormatterExt {
    fn to_kilobytes(&self) -> String;
    fn to_bytes(&self) -> String;
}

impl<T: Numeric> BytesFormatterExt for T {
    fn to_kilobytes(&self) -> String {
        format_kb(*self)
    }

    fn to_bytes(&self) -> String {
        format_bytes(*self)
    }
}

impl BytesFormatterExt for i32 {
    fn to_kilobytes(&self) -> String {
        format_kb(*self as u32)
    }
    fn to_bytes(&self) -> String {
        format_bytes(*self as u32)
    }
}

impl BytesFormatterExt for i64 {
    fn to_kilobytes(&self) -> String {
        format_kb(*self as u64)
    }
    fn to_bytes(&self) -> String {
        format_bytes(*self as u64)
    }
}

pub trait PercentFormatterExt {
    fn to_percent2(&self) -> String;
    fn to_percent0(&self) -> String;
    fn to_percent1(&self) -> String;
}

impl PercentFormatterExt for f64 {
    fn to_percent2(&self) -> String {
        format!("{:.2}%", *self * 100f64)
    }
    fn to_percent0(&self) -> String {
        format!("{:.0}%", *self * 100f64)
    }
    fn to_percent1(&self) -> String {
        format!("{:.1}%", *self * 100f64)
    }
}

pub trait ClampNumExt<T> {
    fn clamp_min(&self, min: T) -> T;
    fn clamp_max(&self, max: T) -> T;
}

impl ClampNumExt<f32> for f32 {
    fn clamp_min(&self, min: f32) -> f32 {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: f32) -> f32 {
        if *self > max {
            max
        } else {
            *self
        }
    }
}

impl ClampNumExt<i32> for i32 {
    fn clamp_min(&self, min: i32) -> i32 {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: i32) -> i32 {
        if *self > max {
            max
        } else {
            *self
        }
    }
}
