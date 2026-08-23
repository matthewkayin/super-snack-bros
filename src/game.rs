use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};

pub const RECT_SIZE: f32 = 32.0;
const RECT_SPEED: f32 = 1.5;

pub struct GameState {
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_direction_x: f32,
    pub rect_direction_y: f32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            rect_x: 10.0,
            rect_y: 10.0,
            rect_direction_x: 1.0,
            rect_direction_y: 1.0
        }
    }

    pub fn update(&mut self) {
        self.rect_x += self.rect_direction_x * RECT_SPEED;
        if self.rect_x <= 0.0 {
            self.rect_x = 0.0;
            self.rect_direction_x = 1.0;
        } else if self.rect_x >= SCREEN_WIDTH - RECT_SIZE {
            self.rect_x = SCREEN_WIDTH - RECT_SIZE;
            self.rect_direction_x = -1.0;
        }

        self.rect_y += self.rect_direction_y * RECT_SPEED;
        if self.rect_y <= 0.0 {
            self.rect_y = 0.0;
            self.rect_direction_y = 1.0;
        } else if self.rect_y >= SCREEN_HEIGHT - RECT_SIZE {
            self.rect_y = SCREEN_HEIGHT - RECT_SIZE;
            self.rect_direction_y = -1.0;
        }
    }
}
