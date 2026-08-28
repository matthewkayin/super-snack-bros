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
        self.player2.update();

        if self.player1.get_center().x < self.player2.get_center().x {
            self.player1.direction = 1.0;
            self.player2.direction = -1.0;
        } else {
            self.player1.direction = -1.0;
            self.player2.direction = 1.0;
        }
    }

    pub fn render(&self) {
        self.player1.render();
        self.player2.render();
    }
}
