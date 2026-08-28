use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::input::*;
use crate::core::rect::Rect;
use crate::core::animation::*;
use crate::core::render::*;
use glam::Vec2;

const FIGHTER_SPAWN_MARGIN: f32 = 32.0;

const FIGHTER_WALK_SPEED: f32 = 2.0;
const FIGHTER_WALK_ACCELERATION: f32 = 0.7;
const FIGHTER_WALK_DECELERATION: f32 = 0.2;

const FIGHTER_GRAVITY: f32 = 0.45;
const FIGHTER_JUMP_ACCELERATION: f32 = -10.0;
const FIGHTER_JUMP_SHORT_HOP_ACCELERATION: f32 = -8.0;
const FIGHTER_FALL_SPEED: f32 = 3.0;

const FIGHTER_JUMP_INPUT_DURATION: u32 = 10;
const FIGHTER_CAYOTE_TIMER: u32 = 10;
const FIGHTER_JUMP_SQUAT_DURATION: u32 = 5;

#[derive(Debug, PartialEq, Eq)]
enum FighterMode {
    Idle,
    JumpSquat
}

pub struct Fighter {
    player: InputPlayer,
    mode: FighterMode,
    animation: AnimationInstance,

    position: Vec2,
    velocity: Vec2,
    direction: i32,

    has_double_jump: bool,
    is_grounded: bool,
    jump_input_timer: u32,
    coyote_timer: u32,
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
            position: Vec2::new(position_x, SCREEN_HEIGHT - render_get_sprite_frame_size(Sprite::Crab).y),
            velocity: Vec2::new(0.0, 0.0),
            direction: if player == InputPlayer::One { 1 } else { -1 },

            has_double_jump: false,
            is_grounded: false,
            jump_input_timer: 0,
            coyote_timer: 0,
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
            if self.can_ground_jump() {
                if !self.can_ground_jump() {
                    self.has_double_jump = false;
                }

                // Begin jump
                self.mode = FighterMode::JumpSquat;
                self.reset_animation();
                self.jump_input_timer = 0;
                self.jump_timer = FIGHTER_JUMP_SQUAT_DURATION;
                self.coyote_timer = 0;
            } else if self.has_double_jump {
                self.jump();
                self.has_double_jump = false;
            } else {
                // Decrement timer
                self.jump_input_timer -= 1;
            }
        }

        // While jumping, move up
        let jumped_this_frame = {
            let mut jumped = false;
            if self.mode == FighterMode::JumpSquat {
                self.jump_timer -= 1;
                if self.jump_timer == 0 {
                    self.mode = FighterMode::Idle;
                    self.jump_timer = 0;
                    self.jump();
                    jumped = true;
                }
            }

            jumped
        };

        // Update direction
        let di = self.get_directional_input();
        if self.is_grounded && di != 0.0 {
            self.direction = di as i32;
        }

        // Movement

        // Turn around on the spot when grounded
        if self.is_grounded &&
            ((di == 1.0 && self.velocity.x < 0.0) ||
            (di == -1.0 && self.velocity.x > 0.0))
        {
            self.velocity.x = 0.0;
        }

        // Deceleration
        if di == 0.0 && self.velocity.x > 0.0 {
            self.velocity.x = (self.velocity.x - FIGHTER_WALK_DECELERATION).max(0.0);
        } else if di == 0.0 && self.velocity.x < 0.0 {
            self.velocity.x = (self.velocity.x + FIGHTER_WALK_DECELERATION).min(0.0);
        }

        // Walk acceleration
        if di == 1.0 && self.velocity.x < FIGHTER_WALK_SPEED {
            self.velocity.x = (self.velocity.x + FIGHTER_WALK_ACCELERATION).min(FIGHTER_WALK_SPEED);
        } else if di == -1.0 && self.velocity.x > -FIGHTER_WALK_SPEED {
            self.velocity.x = (self.velocity.x - FIGHTER_WALK_ACCELERATION).max(-FIGHTER_WALK_SPEED);
        }

        self.velocity.y += FIGHTER_GRAVITY;
        if self.velocity.y > FIGHTER_FALL_SPEED {
            self.velocity.y = FIGHTER_FALL_SPEED;
        }
        self.position += self.velocity;

        // Floor collision
        let was_grounded = self.is_grounded;
        self.is_grounded = false;
        let collide_rect = self.get_collide_rect();
        if self.position.y + collide_rect.position.y + collide_rect.size.y > SCREEN_HEIGHT {
            self.position.y = SCREEN_HEIGHT - collide_rect.size.y - collide_rect.position.y;
            self.is_grounded = true
        }

        // Cayote timer
        if !self.is_grounded && was_grounded && !jumped_this_frame {
            self.coyote_timer = FIGHTER_CAYOTE_TIMER;
        }
        if self.coyote_timer != 0 {
            self.coyote_timer -= 1;
        }

        // Reset jumps remaining
        if self.is_grounded {
            self.has_double_jump = true;
        }

        let message = format!("Velocity Y {}", self.velocity.y);
        web_sys::console::debug_1(&message.into());
    }

    fn can_ground_jump(&self) -> bool {
        (self.is_grounded || self.coyote_timer != 0) &&
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
        if self.mode == FighterMode::JumpSquat {
            return Animation::CrabJump;
        }

        if !self.is_grounded {
            return Animation::CrabFall
        }

        if self.velocity.x != 0.0 {
            return Animation::CrabWalkForward
        }

        Animation::CrabIdle
    }

    fn get_collide_rect(&self) -> Rect {
        Rect {
            position: Vec2::new(9.0, 5.0) * 2.0,
            size: Vec2::new(14.0, 11.0) * 2.0
        }
    }

    fn jump(&mut self) {
        let jump_impulse = if input_is_action_pressed(self.player, InputAction::Up) {
            FIGHTER_JUMP_ACCELERATION
        } else {
            FIGHTER_JUMP_SHORT_HOP_ACCELERATION
        };
        self.velocity.y += jump_impulse;
        self.velocity.y = self.velocity.y.max(FIGHTER_JUMP_ACCELERATION)
    }

    pub fn render(&self) {
        render_sprite(Sprite::Crab, self.position, self.animation.h_frame, self.animation.v_frame, self.direction == -1);
    }
}
