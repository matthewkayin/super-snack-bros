// use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::{animation::{ANIMATION_CRAB_IDLE, AnimationInstance}, constants::SCREEN_HEIGHT};
use glam::Vec2;

const FIGHTER_FALL_SPEED: f32 = 1.0;

struct Rect {
    position: Vec2,
    size: Vec2
}

enum FighterMode {
    Idle,
    Walk
}

pub struct Fighter {
    pub position: Vec2
}

impl Rect {
    fn new(position: Vec2, size: Vec2) -> Self {
        Rect {
            position,
            size
        }
    }

    fn intersects(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.x < other.position.x ||
            other.position.x + other.size.x < self.size.x ||
            self.position.y + self.size.y < other.size.y ||
            other.position.y + other.size.y < self.size.y)
    }
}

impl Fighter {
    fn new() -> Self {
        Fighter {
            position: Vec2::new(10.0, 10.0)
        }
    }

    fn update(&mut self) {
        self.position.y += FIGHTER_FALL_SPEED;

        let collide_rect = self.get_collide_rect();
        if self.position.y + collide_rect.position.y + collide_rect.size.y > SCREEN_HEIGHT {
            self.position.y = SCREEN_HEIGHT - collide_rect.size.y - collide_rect.position.y;
        }
    }

    fn get_collide_rect(&self) -> Rect {
        Rect {
            position: Vec2::new(9.0, 5.0),
            size: Vec2::new(14.0, 11.0)
        }
    }
}

pub struct GameState {
    pub crab_anim: AnimationInstance,
    pub crab: Fighter,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            crab_anim: ANIMATION_CRAB_IDLE.instance(),
            crab: Fighter::new()
        }
    }

    pub fn update(&mut self) {
        self.crab_anim.update();
        self.crab.update();
    }
}
