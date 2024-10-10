pub fn apply_scroll(text: &String, scroll: i32) -> String {
    if scroll <= 0 {
        return text.clone();
    }
    if scroll as usize >= text.len() {
        return String::new();
    }
    text.chars().skip(scroll as usize).collect()
}

pub fn contains_all_words(text: &str, words: &Vec<String>) -> bool {
    let lower_text = text.to_lowercase();
    words.iter().all(|it| lower_text.contains(it))
}
