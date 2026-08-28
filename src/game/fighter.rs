use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::input::*;
use crate::core::rect::Rect;
use crate::core::animation::*;
use crate::core::render::*;
use glam::Vec2;

const FIGHTER_WALK_SPEED: f32 = 1.0;
const FIGHTER_FALL_SPEED: f32 = 2.0;
const FIGHTER_SPAWN_MARGIN: f32 = 32.0;

const FIGHTER_JUMP_INPUT_DURATION: u32 = 5;
const FIGHTER_CAYOTE_TIMER: u32 = 5;
const FIGHTER_JUMP_MIN_DURATION: u32 = 5;
const FIGHTER_JUMP_DURATION: u32 = 30;
const FIGHTER_JUMP_SPEED: f32 = -4.0;

#[derive(Debug, PartialEq, Eq)]
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

    is_grounded: bool,
    jump_input_timer: u32,
    cayote_timer: u32,
    jump_timer: u32
}

impl Fighter {
    pub fn new(player: InputPlayer) -> Self {
        let position_x = if player == InputPlayer::One {
            FIGHTER_SPAWN_MARGIN
        } else {
            SCREEN_WIDTH - FIGHTER_SPAWN_MARGIN - render_get_sprite_frame_size(Sprite::Crab).x
        };
        Fighter {
            player,
            mode: FighterMode::Idle,
            animation: Animation::CrabIdle.instance(),
            position: Vec2::new(position_x, 0.0),
            direction: 1.0,

            is_grounded: false,
            jump_input_timer: 0,
            cayote_timer: 0,
            jump_timer: 0
        }
    }

    pub fn update(&mut self) {
        // Animation
        if self.animation.name != self.get_expected_animation() {
            self.reset_animation();
        }
        self.animation.update();

        // JUMPING

        // On up press, start jump timer
        if input_is_action_just_pressed(self.player, InputAction::Up) {
            self.jump_input_timer = FIGHTER_JUMP_INPUT_DURATION;
        }

        // When jump timer is non-zero and we can jump, begin jump
        if self.jump_input_timer != 0 {
            if self.can_jump() {
                // Begin jump
                self.mode = FighterMode::JumpBegin;
                self.reset_animation();
                self.jump_input_timer = 0;
            } else {
                // Decrement timer
                self.jump_input_timer -= 1;
            }
        }

        // While beginning jumping, if the animation is finished, jump
        if self.mode == FighterMode::JumpBegin {
            self.mode = FighterMode::JumpContinue;
            self.reset_animation();
            self.jump_timer = FIGHTER_JUMP_DURATION;
        }

        // While jumping, move up
        if self.mode == FighterMode::JumpContinue {
            if !input_is_action_pressed(self.player, InputAction::Up) &&
                self.jump_timer < FIGHTER_JUMP_DURATION - FIGHTER_JUMP_MIN_DURATION
            {
                self.jump_timer = 0;
            } else {
                self.jump_timer -= 1;
            }

            if self.jump_timer == 0 {
                self.mode = FighterMode::Idle;
                self.reset_animation();
            }
        }

        // Movement
        let velocity_y = if self.mode == FighterMode::JumpContinue {
            FIGHTER_JUMP_SPEED
        } else {
            FIGHTER_FALL_SPEED
        };
        self.position += Vec2::new(self.get_directional_input() * FIGHTER_WALK_SPEED, velocity_y);

        // Floor collision
        let was_grounded = self.is_grounded;
        self.is_grounded = false;
        let collide_rect = self.get_collide_rect();
        if self.position.y + collide_rect.position.y + collide_rect.size.y > SCREEN_HEIGHT {
            self.position.y = SCREEN_HEIGHT - collide_rect.size.y - collide_rect.position.y;
            self.is_grounded = true
        }

        // Cayote timer
        if !self.is_grounded && was_grounded {
            self.cayote_timer = FIGHTER_CAYOTE_TIMER;
        }
        if self.cayote_timer != 0 {
            self.cayote_timer -= 1;
        }
    }

    fn can_jump(&self) -> bool {
        (self.is_grounded || self.cayote_timer != 0) &&
        self.mode == FighterMode::Idle
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

    fn reset_animation(&mut self) {
        self.animation = self.get_expected_animation().instance();
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

    pub fn get_center(&self) -> Vec2 {
        self.position + (render_get_sprite_frame_size(Sprite::Crab) / 2.0)
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
