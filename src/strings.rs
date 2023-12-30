pub fn apply_scroll(text: &String, scroll: i32) -> String {
    if scroll <= 0 {
        return text.clone();
    }
    if scroll as usize >= text.len() {
        return String::new();
    }
    text.chars().skip(scroll as usize).collect()
}
