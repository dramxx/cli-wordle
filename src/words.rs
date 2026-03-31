pub fn load_words(lang: &str) -> Vec<String> {
    let words = match lang {
        "sk" => include_str!("../sk.txt"),
        _ => include_str!("../en.txt"),
    };
    words
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() == 5)
        .collect()
}
