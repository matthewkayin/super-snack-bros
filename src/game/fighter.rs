use crate::constants::SCREEN_HEIGHT;
use crate::core::input::*;
use crate::core::rect::Rect;
use crate::core::animation::*;
use crate::core::render::*;
use glam::Vec2;

const FIGHTER_WALK_SPEED: f32 = 1.0;
const FIGHTER_FALL_SPEED: f32 = 2.0;

enum FighterPlayer {
    One,
    Two
}

#[derive(Debug)]
enum FighterMode {
    Idle,
    JumpBegin,
    JumpContinue
}

pub struct Fighter {
    player: InputPlayer,
    mode: FighterMode,
    pub animation: AnimationInstance,
    pub position: Vec2,
    pub direction: f32,
    is_grounded: bool
}

impl Fighter {
    pub fn new(player: InputPlayer) -> Self {
        Fighter {
            player,
            mode: FighterMode::Idle,
            animation: Animation::CrabIdle.instance(),
            position: Vec2::new(0.0, 0.0),
            direction: 1.0,
            is_grounded: false
        }
    }

    pub fn update(&mut self) {
        // Animation
        if self.animation.name != self.get_expected_animation() {
            self.animation = self.get_expected_animation().instance();
        }
        self.animation.update();

        // Movement
        self.position += Vec2::new(self.get_directional_input() * FIGHTER_WALK_SPEED, FIGHTER_FALL_SPEED);

        // Floor collision
        self.is_grounded = false;
        let collide_rect = self.get_collide_rect();
        if self.position.y + collide_rect.position.y + collide_rect.size.y > SCREEN_HEIGHT {
            self.position.y = SCREEN_HEIGHT - collide_rect.size.y - collide_rect.position.y;
            self.is_grounded = true
        }
    }

    fn get_directional_input(&self) -> f32 {
        if input_is_action_pressed(self.player, InputAction::Right) {
            1.0
        } else if input_is_action_pressed(self.player, InputAction::Left) {
            -1.0
        } else {
            0.0
        }
    }

    fn get_expected_animation(&self) -> Animation {
        match self.mode {
            FighterMode::Idle => {
                if !self.is_grounded {
                    return Animation::CrabFall
                }

                let di = self.get_directional_input();
                if di == self.direction {
                    return Animation::CrabWalkForward
                }
                if di == -self.direction {
                    return Animation::CrabWalkBackward
                }
                Animation::CrabIdle
            },
            FighterMode::JumpBegin => Animation::CrabJumpBegin,
            FighterMode::JumpContinue => Animation::CrabFall,
        }
    }

    fn get_collide_rect(&self) -> Rect {
        Rect {
            position: Vec2::new(9.0, 5.0) * 2.0,
            size: Vec2::new(14.0, 11.0) * 2.0
        }
    }

    pub fn render(&self) {
        render_sprite(Sprite::Crab, self.position, self.animation.h_frame, self.animation.v_frame, self.direction == -1.0);
    }
}
