use crate::constants::*;
use crate::core::input::*;
use crate::game::collider::*;
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
const FIGHTER_JUMPED_ON_ACCELERATION: f32 = 2.0;

const FIGHTER_JUMP_INPUT_DURATION: u32 = 10;
const FIGHTER_COYOTE_TIMER_DURATION: u32 = 10;
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
    pub velocity: Vec2,
    direction: i32,

    jumped_last_frame: bool,
    has_double_jump: bool,
    is_grounded: bool,
    was_grounded: bool,
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
            position: Vec2::new(position_x, SCREEN_HEIGHT - render_get_sprite_frame_size(Sprite::Crab).y - 100.0),
            velocity: Vec2::new(0.0, 0.0),
            direction: if player == InputPlayer::One { 1 } else { -1 },

            jumped_last_frame: false,
            has_double_jump: false,
            is_grounded: false,
            was_grounded: false,
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

        // Coyote timer
        if !self.is_grounded && self.was_grounded && !self.jumped_last_frame {
            self.coyote_timer = FIGHTER_COYOTE_TIMER_DURATION;
        }
        if self.coyote_timer != 0 {
            self.coyote_timer -= 1;
        }

        // Reset jumps remaining
        if self.is_grounded {
            self.has_double_jump = true;
        }

        self.jumped_last_frame = false;

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
        if self.mode == FighterMode::JumpSquat {
            self.jump_timer -= 1;
            if self.jump_timer == 0 {
                self.mode = FighterMode::Idle;
                self.jump_timer = 0;
                self.jump();
                self.jumped_last_frame = true;
            }
        }

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

        self.was_grounded = self.is_grounded;
        self.is_grounded = false;
    }

    pub fn handle_static_collisions(&mut self, static_colliders: &Vec<Collider>) {
        for collider in static_colliders.iter() {
            let collision = self.get_pushbox().get_collision(collider);
            self.position += collision;
            if collision.y < 0.0 {
                self.is_grounded = true;
                self.velocity.y = 0.0;
            }
        }
    }

    pub fn handle_pushbox_collision(&mut self, collision: Vec2) {
        if self.was_grounded {
            self.position.x += collision.x * 0.5;
        }
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
            return Animation::CrabWalk
        }

        Animation::CrabIdle
    }

    pub fn get_pushbox(&self) -> Collider {
        Collider::new(self.position + Vec2::new(9.0, 5.0), Vec2::new(14.0, 11.0))
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
