use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::animation::*;
use crate::core::input::*;
use crate::core::render::*;
use glam::Vec2;

// RECT

struct Rect {
    position: Vec2,
    size: Vec2
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

// FIGHTER

const FIGHTER_WALK_SPEED: f32 = 1.0;
const FIGHTER_FALL_SPEED: f32 = 1.0;

enum FighterMode {
    Idle,
    Walk
}

pub struct Fighter {
    mode: FighterMode,
    pub animation: AnimationInstance,
    pub position: Vec2
}

impl Fighter {
    fn new() -> Self {
        Fighter {
            mode: FighterMode::Idle,
            animation: Animation::CrabIdle.instance(),
            position: Vec2::new(10.0, 10.0)
        }
    }

    fn update(&mut self) {
    }

    fn get_expected_animation(&self) -> &Animation {
        match self.mode {
            FighterMode::Idle => &Animation::CrabIdle,
            FighterMode::Walk => &Animation::CrabWalk,
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
            crab_anim: Animation::CrabIdle.instance(),
            crab: Fighter::new()
        }
    }

    pub fn update(&mut self) {
        self.crab_anim.update();
        if input_is_action_pressed(InputAction::PlayerOneRight) {
            self.crab.position.x += FIGHTER_WALK_SPEED;
        }
        self.crab.position.y += FIGHTER_FALL_SPEED;

        let collide_rect = self.crab.get_collide_rect();
        if self.crab.position.y + collide_rect.position.y + collide_rect.size.y > SCREEN_HEIGHT {
            self.crab.position.y = SCREEN_HEIGHT - collide_rect.size.y - collide_rect.position.y;
        }
    }

    pub fn render(&self) {
        render_sprite(Sprite::Crab, self.crab.position, self.crab_anim.h_frame, self.crab_anim.v_frame);
    }
}
