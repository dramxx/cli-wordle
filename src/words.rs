pub fn load_words() -> Vec<String> {
    include_str!("../words.txt")
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() == 5)
        .collect()
}
