use std::time::Instant;

use crate::game::{Game, GameStatus};

const CURSOR_BLINK_MS: u128 = 500;
const MESSAGE_DURATION_MS: u128 = 1500;
const SHAKE_DURATION_MS: u128 = 300;

pub enum Screen {
    Playing,
    Won,
    Lost,
    Quit,
}

pub struct App {
    pub game: Game,
    pub screen: Screen,
    pub words: Vec<String>,
    pub shake: Option<Instant>,
    pub msg: Option<String>,
    pub msg_time: Option<Instant>,
    pub cursor_blink: bool,
    pub last_tick: Instant,
}

impl App {
    pub fn new(words: Vec<String>) -> Self {
        let game = Game::new(words.clone());
        Self {
            screen: Screen::Playing,
            words,
            game,
            shake: None,
            msg: None,
            msg_time: None,
            cursor_blink: false,
            last_tick: Instant::now(),
        }
    }

    pub fn type_char(&mut self, c: char) {
        if matches!(self.screen, Screen::Playing) {
            self.game.type_char(c);
        }
    }

    pub fn backspace(&mut self) {
        if matches!(self.screen, Screen::Playing) {
            self.game.backspace();
        }
    }

    pub fn submit(&mut self) {
        if !matches!(self.screen, Screen::Playing) {
            return;
        }

        match self.game.submit() {
            Ok(()) => {
                self.update_screen();
            }
            Err(e) => {
                self.shake = Some(Instant::now());
                self.set_message(e);
            }
        }
    }

    fn update_screen(&mut self) {
        self.screen = match self.game.status {
            GameStatus::Playing => Screen::Playing,
            GameStatus::Won => Screen::Won,
            GameStatus::Lost => Screen::Lost,
        };
    }

    pub fn new_game(&mut self) {
        self.game = Game::new(self.words.clone());
        self.screen = Screen::Playing;
        self.shake = None;
        self.msg = None;
        self.msg_time = None;
    }

    pub fn quit(&mut self) {
        self.screen = Screen::Quit;
    }

    pub fn is_done(&self) -> bool {
        !matches!(self.screen, Screen::Playing)
    }

    fn set_message(&mut self, msg: String) {
        self.msg = Some(msg);
        self.msg_time = Some(Instant::now());
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_tick).as_millis() >= CURSOR_BLINK_MS {
            self.cursor_blink = !self.cursor_blink;
            self.last_tick = now;
        }

        if let Some(msg_time) = self.msg_time {
            if now.duration_since(msg_time).as_millis() > MESSAGE_DURATION_MS {
                self.msg = None;
                self.msg_time = None;
            }
        }

        if let Some(shake_time) = self.shake {
            if now.duration_since(shake_time).as_millis() >= SHAKE_DURATION_MS {
                self.shake = None;
            }
        }
    }
}
