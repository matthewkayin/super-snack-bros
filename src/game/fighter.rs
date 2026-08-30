use crate::core::input::*;
use crate::game::rect::*;
use crate::core::animation::*;
use crate::core::render::*;
use glam::Vec2;

const FIGHTER_WALK_SPEED: f32 = 1.5;
const FIGHTER_WALK_ACCELERATION: f32 = 0.55;
const FIGHTER_WALK_DECELERATION: f32 = 0.15;

const FIGHTER_GRAVITY: f32 = 0.34;
const FIGHTER_JUMP_ACCELERATION: f32 = -6.0;
const FIGHTER_JUMP_SHORT_HOP_ACCELERATION: f32 = -4.0;
const FIGHTER_FALL_SPEED: f32 = 2.25;

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
    pub sprite: Sprite,
    mode: FighterMode,
    animation: AnimationInstance,

    pub position: Vec2,
    pub velocity: Vec2,
    direction: i32,

    has_double_jump: bool,
    is_grounded: bool,
    jump_input_timer: u32,
    coyote_timer: u32,
    jump_timer: u32
}

impl Fighter {
    pub fn new(player: InputPlayer) -> Self {
        Fighter {
            player,
            sprite: match player {
                InputPlayer::One => Sprite::CrabOrange,
                InputPlayer::Two => Sprite::CrabGreen,
            },
            mode: FighterMode::Idle,

            animation: Animation::CrabIdle.instance(),
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            direction: if player == InputPlayer::One { 1 } else { -1 },

            has_double_jump: false,
            is_grounded: false,
            jump_input_timer: 0,
            coyote_timer: 0,
            jump_timer: 0
        }
    }

    pub fn update(&mut self, colliders: &Vec<Rect>) {
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
        let mut jumped_this_frame = false;
        if self.mode == FighterMode::JumpSquat {
            self.jump_timer -= 1;
            if self.jump_timer == 0 {
                self.mode = FighterMode::Idle;
                self.jump_timer = 0;
                self.jump();
                jumped_this_frame = true;
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

        let was_grounded = self.is_grounded;
        self.is_grounded = false;

        // Move X
        if self.velocity.x != 0.0 {
            let old_pushbox = self.get_pushbox();
            self.position.x += self.velocity.x;
            let pushbox = self.get_pushbox();

            for collider in colliders.iter() {
                // First, check that we are aligned on the y axis
                let vertically_overlapping = !(
                    pushbox.position.y + pushbox.size.y <= collider.position.y ||
                    pushbox.position.y >= collider.position.y + collider.size.y);

                if vertically_overlapping && self.velocity.y >= 0.0 {
                    if self.velocity.x > 0.0 &&
                        old_pushbox.position.x <= collider.position.x &&
                        pushbox.position.x + pushbox.size.x > collider.position.x
                    {
                        self.position.x += collider.position.x - (pushbox.position.x + pushbox.size.x);
                        self.velocity.x = 0.0;
                        break;
                    }

                    if self.velocity.x < 0.0 &&
                        old_pushbox.position.x >= collider.position.x + collider.size.x &&
                        pushbox.position.x < collider.position.x + collider.size.x
                    {
                        self.position.x += (collider.position.x + collider.size.x) - pushbox.position.x;
                        self.velocity.x = 0.0;
                        break;
                    }
                }
            }
        }

        // Move Y
        if self.velocity.y != 0.0 {
            let old_pushbox = self.get_pushbox();
            self.position.y += self.velocity.y;
            let pushbox = self.get_pushbox();

            for collider in colliders.iter() {
                if pushbox.intersects_horizontally(collider) {
                    if self.velocity.y > 0.0 &&
                        old_pushbox.position.y <= collider.position.y &&
                        pushbox.position.y + pushbox.size.y > collider.position.y
                    {
                        self.position.y += collider.position.y - (pushbox.position.y + pushbox.size.y);
                        self.velocity.y = 0.0;
                        self.is_grounded = true;
                        break;
                    }

                    if self.velocity.y < 0.0 &&
                        old_pushbox.position.y >= collider.position.y + collider.size.y &&
                        pushbox.position.y < collider.position.y + collider.size.y
                    {
                        // self.position.y += (collider.position.y + collider.size.y) - pushbox.position.y;
                        // self.velocity.y = 0.0;
                        break;
                    }
                }
            }
        }

        // Coyote timer
        if !self.is_grounded && was_grounded && !jumped_this_frame {
            self.coyote_timer = FIGHTER_COYOTE_TIMER_DURATION;
        }
        if self.coyote_timer != 0 {
            self.coyote_timer -= 1;
        }

        // Reset jumps remaining
        if self.is_grounded {
            self.has_double_jump = true;
        }
    }

    pub fn handle_pushbox_collision(&mut self, collision: Vec2) {
        if self.is_grounded {
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

    pub fn get_pushbox(&self) -> Rect {
        Rect {
            position: self.position + Vec2::new(9.0, 5.0),
            size: Vec2::new(14.0, 11.0)
        }
    }

    fn jump(&mut self) {
        self.velocity.y = if self.can_ground_jump() && !input_is_action_pressed(self.player, InputAction::Up) {
            FIGHTER_JUMP_SHORT_HOP_ACCELERATION
        } else {
            FIGHTER_JUMP_ACCELERATION
        };
    }

    pub fn render(&self) {
        render_sprite(self.sprite, self.position, self.animation.h_frame, self.animation.v_frame, self.direction == -1);
    }
}
