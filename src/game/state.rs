use crate::game::fighter::*;
use crate::core::input::*;

pub struct GameState {
    pub player1: Fighter,
    pub player2: Fighter
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player1: Fighter::new(InputPlayer::One),
            player2: Fighter::new(InputPlayer::Two),
        }
    }

    pub fn update(&mut self) {
        self.player1.update();
        // self.player2.update();
    }

    pub fn render(&self) {
        self.player1.render();
        // self.player2.render();
    }
}
