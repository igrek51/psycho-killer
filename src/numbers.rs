use std::fmt::Display;

pub trait Numeric: Into<u64> + Display + Copy {}
impl Numeric for u16 {}
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
    fn to_percent1(&self) -> String;
    fn to_percent_len5(&self) -> String;
}

impl PercentFormatterExt for f64 {
    fn to_percent2(&self) -> String {
        format!("{:.2}%", *self * 100f64)
    }
    fn to_percent1(&self) -> String {
        format!("{:.1}%", *self * 100f64)
    }
    fn to_percent_len5(&self) -> String {
        if *self >= 1f64 {
            format!("{:.0}%", *self * 100f64)
        } else {
            format!("{:.1}%", *self * 100f64)
        }
    }
}

// Clamp extensions
pub trait ClampNumType: PartialOrd + Copy {}
impl ClampNumType for usize {}
impl ClampNumType for u16 {}
impl ClampNumType for u32 {}
impl ClampNumType for u64 {}
impl ClampNumType for i32 {}
impl ClampNumType for i64 {}
impl ClampNumType for f32 {}
impl ClampNumType for f64 {}

pub trait ClampNumExt<T> {
    fn clamp_min(&self, min: T) -> T;
    fn clamp_max(&self, max: T) -> T;
}

impl<T: ClampNumType> ClampNumExt<T> for T {
    fn clamp_min(&self, min: T) -> T {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: T) -> T {
        if *self > max {
            max
        } else {
            *self
        }
    }
}

// Extensions for generic numeric operations for different types
#[allow(dead_code)]
pub trait ConvertibleIntExt<T>: PartialOrd + Copy + PartialEq + Eq {
    fn into_intermediary(&self) -> i32;
    fn from_intermediary(intermediary: i32) -> T;
}

impl ConvertibleIntExt<u16> for u16 {
    fn into_intermediary(&self) -> i32 {
        *self as i32
    }
    fn from_intermediary(intermediary: i32) -> u16 {
        intermediary as u16
    }
}

impl ConvertibleIntExt<usize> for usize {
    fn into_intermediary(&self) -> i32 {
        *self as i32
    }
    fn from_intermediary(intermediary: i32) -> usize {
        intermediary as usize
    }
}

impl ConvertibleIntExt<i32> for i32 {
    fn into_intermediary(&self) -> i32 {
        *self
    }
    fn from_intermediary(intermediary: i32) -> i32 {
        intermediary
    }
}

#[allow(dead_code)]
pub trait MyIntExt<T> {
    fn move_rotating(&self, delta: i32, max: T) -> T;
    fn move_bound(&self, delta: i32, max: T) -> T;
    fn add_casting(&self, delta: i32) -> i32;
    fn fraction(&self, multiplier: f64) -> T;
    fn clamp_usize(&self) -> usize;
}

impl<T: ConvertibleIntExt<T>> MyIntExt<T> for T {
    fn move_rotating(&self, delta: i32, max: T) -> T {
        let max_i32: i32 = max.into_intermediary();
        if max_i32 == 0 {
            return T::from_intermediary(0);
        }
        let self_i32: i32 = (*self).into_intermediary();
        let mut new_cursor: i32 = self_i32 + delta;
        while new_cursor < 0 {
            new_cursor += max_i32;
        }
        T::from_intermediary(new_cursor % max_i32)
    }

    fn move_bound(&self, delta: i32, max: T) -> T {
        let max_i32: i32 = max.into_intermediary();
        let self_i32: i32 = (*self).into_intermediary();
        T::from_intermediary((self_i32 + delta).clamp_max(max_i32 - 1).clamp_min(0))
    }

    fn add_casting(&self, delta: i32) -> i32 {
        let self_i32: i32 = (*self).into_intermediary();
        self_i32 + delta
    }

    fn fraction(&self, multiplier: f64) -> T {
        let self_i32: i32 = (*self).into_intermediary();
        let multiplied = (self_i32 as f64) * multiplier;
        T::from_intermediary(multiplied as i32)
    }

    fn clamp_usize(&self) -> usize {
        let self_i32: i32 = (*self).into_intermediary();
        self_i32.clamp_min(0) as usize
    }
}
