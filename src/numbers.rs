/// Format kilobytes as kiB, MiB or GiB, with 2-digit precision
pub fn format_kbs(kilobytes: u64) -> String {
    if kilobytes < 1024 {
        format!("{kilobytes} KiB")
    } else if kilobytes < 1024 * 1024 {
        format!("{:.2} MiB", kilobytes as f64 / 1024f64)
    } else {
        format!("{:.2} GiB", kilobytes as f64 / 1024f64 / 1024f64)
    }
}

pub trait BytesFormatterExt {
    fn to_bytes(&self) -> String;
}

impl BytesFormatterExt for u64 {
    fn to_bytes(&self) -> String {
        format_kbs(*self)
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
