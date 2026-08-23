// use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::animation::{ANIMATION_CRAB_IDLE, AnimationInstance};

pub struct GameState {
    pub crab_anim: AnimationInstance
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            crab_anim: ANIMATION_CRAB_IDLE.instance()
        }
    }

    pub fn update(&mut self) {
        self.crab_anim.update();
    }
}
