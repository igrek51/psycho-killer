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

pub fn first_cmd_part(cmd: &str) -> String {
    if cmd.starts_with("\"") {
        return cmd.chars().skip(1).take_while(|c| *c != '"').collect();
    }
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }
    parts[0].to_string()
}
