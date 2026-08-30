use crate::core::input::*;
use crate::game::rect::*;
use crate::core::animation::*;
use crate::core::render::*;
use std::collections::VecDeque;
use glam::Vec2;

const FIGHTER_WALK_SPEED: f32 = 1.5;
const FIGHTER_WALK_ACCELERATION: f32 = 0.55;
const FIGHTER_WALK_DECELERATION: f32 = 0.15;

const FIGHTER_GRAVITY: f32 = 0.34;
const FIGHTER_JUMP_ACCELERATION: f32 = -6.0;
const FIGHTER_JUMP_SHORT_HOP_ACCELERATION: f32 = -4.0;
const FIGHTER_FALL_SPEED: f32 = 2.25;

const FIGHTER_COYOTE_TIMER_DURATION: u32 = 10;
const FIGHTER_JUMP_SQUAT_DURATION: u32 = 5;

const FIGHTER_INPUT_TTL: u32 = 8;
const FIGHTER_INPUT_QUEUE_MAX_SIZE: usize = 4;

#[derive(Debug, PartialEq, Eq)]
enum FighterMode {
    Idle,
    JumpSquat,
    PunchGround1,
    PunchGround2
}

#[derive(PartialEq, Eq)]
enum FighterInputType {
    Jump,
    Punch
}

struct FighterInput {
    typ: FighterInputType,
    ttl: u32
}

pub struct Fighter {
    player: InputPlayer,
    pub sprite: Sprite,
    mode: FighterMode,
    animation: AnimationInstance,
    input_queue: VecDeque<FighterInput>,

    pub position: Vec2,
    pub velocity: Vec2,
    direction: i32,

    has_double_jump: bool,
    is_grounded: bool,
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
            input_queue: VecDeque::new(),

            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            direction: if player == InputPlayer::One { 1 } else { -1 },

            has_double_jump: false,
            is_grounded: false,
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

        // CHECK INPUTS
        if input_is_action_just_pressed(self.player, InputAction::Up) {
            self.queue_input(FighterInputType::Jump);
        }
        if input_is_action_just_pressed(self.player, InputAction::A) {
            self.queue_input(FighterInputType::Punch);
        }

        // INPUT QUEUE
        for input in self.input_queue.iter_mut() {
            input.ttl -= 1;
        }
        while !self.input_queue.is_empty() && self.input_queue.front().unwrap().ttl == 0 {
            self.input_queue.pop_front();
        }
        match self.input_queue.front() {
            Some(input) => {
                match input.typ {
                    FighterInputType::Jump => {
                        self.handle_input_jump();
                    },
                    FighterInputType::Punch => {
                        self.handle_input_punch();
                    }
                }
                self.input_queue.pop_front();
            },
            None => ()
        }

        // UPDATE

        let mut jumped_this_frame = false;
        let di = self.get_directional_input();

        match self.mode {
            FighterMode::Idle => {
                // Update direction
                if self.is_grounded && di != 0.0 {
                    self.direction = di as i32;
                }
            },
            FighterMode::JumpSquat => {
                // Count jump squat time
                self.jump_timer -= 1;
                if self.jump_timer == 0 {
                    self.mode = FighterMode::Idle;
                    self.jump_timer = 0;
                    self.jump();
                    jumped_this_frame = true;
                }
            },
            FighterMode::PunchGround1 | FighterMode::PunchGround2 => {
                if self.animation.is_finished() {
                    self.mode = FighterMode::Idle;
                }
            }
        }

        // MOVE

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
        if self.mode == FighterMode::PunchGround1 || self.mode == FighterMode::PunchGround2 {
            self.velocity.x = 0.0;
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

    fn queue_input(&mut self, typ: FighterInputType) {
        if self.input_queue.len() == FIGHTER_INPUT_QUEUE_MAX_SIZE {
            self.input_queue.pop_front();
        }
        self.input_queue.push_back(FighterInput {
            typ,
            ttl: FIGHTER_INPUT_TTL
        });
    }

    fn handle_input_jump(&mut self) {
        if self.can_ground_jump() {
            // Begin jump
            self.mode = FighterMode::JumpSquat;
            self.reset_animation();
            self.jump_timer = FIGHTER_JUMP_SQUAT_DURATION;
            self.coyote_timer = 0;
        } else if self.has_double_jump {
            self.jump();
            self.has_double_jump = false;
        }
    }

    fn handle_input_punch(&mut self) {
        if self.is_grounded {
            if self.mode == FighterMode::Idle {
                self.mode = FighterMode::PunchGround1;
                self.reset_animation();
            } else if self.mode == FighterMode::PunchGround1 && self.animation.is_on_last_frame() {
                self.mode = FighterMode::PunchGround2;
                self.reset_animation();
            }
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
        if self.mode == FighterMode::PunchGround1 || self.mode == FighterMode::PunchGround2 {
            return Animation::CrabPunch;
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

    // Receives damage
    pub fn get_hurtbox(&self) -> Rect {
        Rect {
            position: self.position + Vec2::new(9.0, 5.0),
            size: Vec2::new(14.0, 11.0)
        }
    }

    // Deals damage
    pub fn get_hitbox(&self) -> Rect {
        match self.mode {
            FighterMode::PunchGround1 | FighterMode::PunchGround2 => Rect {
                position: self.position + Vec2::new(20.0, 3.0),
                size: Vec2::new(12.0, 7.0)
            },
            _ => Rect {
                position: Vec2::ZERO,
                size: Vec2::ZERO
            }
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
