use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LetterState {
    Correct,
    Present,
    Absent,
    Unknown,
}

#[derive(Clone)]
pub struct Guess {
    pub letters: [char; 5],
    pub states: [LetterState; 5],
}

pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

pub struct Game {
    pub target: [char; 5],
    pub guesses: Vec<Guess>,
    pub current: Vec<char>,
    pub keyboard: HashMap<char, LetterState>,
    pub status: GameStatus,
    pub word_list: Vec<String>,
}

impl Game {
    pub fn new(words: Vec<String>) -> Self {
        let target_str = words
            .choose(&mut rand::thread_rng())
            .expect("word list is empty")
            .to_uppercase();
        let target: [char; 5] = target_str.chars().collect::<Vec<_>>().try_into().unwrap();
        let keyboard = Self::init_keyboard();

        Self {
            target,
            guesses: Vec::new(),
            current: Vec::new(),
            keyboard,
            status: GameStatus::Playing,
            word_list: words,
        }
    }

    fn init_keyboard() -> HashMap<char, LetterState> {
        let mut keyboard = HashMap::new();
        for c in b'A'..=b'Z' {
            keyboard.insert(c as char, LetterState::Unknown);
        }
        keyboard
    }

    pub fn type_char(&mut self, c: char) {
        if self.current.len() < 5 && matches!(self.status, GameStatus::Playing) {
            self.current.push(c.to_ascii_uppercase());
        }
    }

    pub fn backspace(&mut self) {
        if matches!(self.status, GameStatus::Playing) {
            self.current.pop();
        }
    }

    pub fn submit(&mut self) -> Result<(), String> {
        if self.current.len() != 5 {
            return Err("Not enough letters".to_string());
        }

        // Use uppercase for display/comparison, lowercase for word list lookup
        let word_upper: String = self.current.iter().collect();
        let word_lower = word_upper.to_lowercase();

        if !self.word_list.contains(&word_lower) {
            return Err("Not in word list".to_string());
        }

        let letters: [char; 5] = self.current.clone().try_into().unwrap();
        let states = score_guess(&self.target, &letters);

        for (i, &c) in letters.iter().enumerate() {
            let current_best = self
                .keyboard
                .get(&c)
                .copied()
                .unwrap_or(LetterState::Unknown);
            if states[i] > current_best {
                self.keyboard.insert(c, states[i]);
            }
        }

        let guess = Guess { letters, states };
        self.guesses.push(guess);
        self.current.clear();

        if word_upper
            .chars()
            .zip(self.target.iter())
            .all(|(a, b)| a == *b)
        {
            self.status = GameStatus::Won;
        } else if self.guesses.len() >= 6 {
            self.status = GameStatus::Lost;
        }

        Ok(())
    }

    pub fn get_target_word(&self) -> String {
        self.target.iter().collect()
    }
}

fn score_guess(target: &[char; 5], guess: &[char; 5]) -> [LetterState; 5] {
    let mut states = [LetterState::Absent; 5];
    let mut target_used = [false; 5];
    let mut guess_used = [false; 5];

    for i in 0..5 {
        if guess[i] == target[i] {
            states[i] = LetterState::Correct;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    for i in 0..5 {
        if guess_used[i] {
            continue;
        }
        for j in 0..5 {
            if !target_used[j] && guess[i] == target[j] {
                states[i] = LetterState::Present;
                target_used[j] = true;
                break;
            }
        }
    }

    states
}

impl PartialOrd for LetterState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LetterState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let rank = |s: &LetterState| match s {
            LetterState::Correct => 3,
            LetterState::Present => 2,
            LetterState::Absent => 1,
            LetterState::Unknown => 0,
        };
        rank(self).cmp(&rank(other))
    }
}
