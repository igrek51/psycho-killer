/// Format kilobytes as kiB, MiB or GiB, with 2-digit precision
pub fn format_kib(kilobytes: u64) -> String {
    if kilobytes < 1024 {
        format!("{kilobytes} KiB")
    } else if kilobytes < 1024 * 1024 {
        format!("{:.2} MiB", kilobytes as f64 / 1024f64)
    } else {
        format!("{:.2} GiB", kilobytes as f64 / 1024f64 / 1024f64)
    }
}

pub trait BytesFormatterExt {
    fn format_kib(&self) -> String;
}

impl BytesFormatterExt for u64 {
    fn format_kib(&self) -> String {
        format_kib(*self)
    }
}

pub trait PercentFormatterExt {
    fn format_percent(&self) -> String;
}

impl PercentFormatterExt for f64 {
    fn format_percent(&self) -> String {
        format!("{:.2}%", *self * 100f64)
    }
}
