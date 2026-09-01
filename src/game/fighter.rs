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
const FIGHTER_AIR_ACCELERATION: f32 = 1.0;
const FIGHTER_FALL_SPEED: f32 = 2.25;

const FIGHTER_COYOTE_TIMER_DURATION: u32 = 10;

const FIGHTER_INPUT_TTL: u32 = 8;
const FIGHTER_INPUT_QUEUE_MAX_SIZE: usize = 4;

#[derive(Debug, PartialEq, Eq)]
enum FighterMode {
    Idle,
    Hitstun,
    Neutral1,
    Neutral2,
    Neutral3
}

#[repr(i8)]
#[derive(PartialEq, Eq, Copy, Clone)]
enum FighterDirection {
    Right = 1,
    Left = -1
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum FighterInputType {
    Jump,
    Punch
}

struct FighterInput {
    typ: FighterInputType,
    ttl: u32
}

#[derive(Debug)]
pub struct FighterHitInfo {
    pub damage: f32,
    pub knockback_strength: f32,
    pub knockback_direction: Vec2,
    pub hitbox: Rect
}

pub struct Fighter {
    player: InputPlayer,
    mode: FighterMode,
    animation: AnimationInstance,
    input_queue: VecDeque<FighterInput>,

    pub sprite: Sprite,
    sprite_frame_size: Vec2,

    pub position: Vec2,
    pub velocity: Vec2,
    direction: FighterDirection,

    // Jump
    jumped_this_frame: bool,
    has_double_jump: bool,
    is_grounded: bool,
    coyote_timer: u32,

    // Hit
    hitstun_timer: u32,
    pub damage: f32,
    pub has_hit: bool,
}

impl Fighter {
    pub fn new(player: InputPlayer) -> Self {
        let sprite = match player {
            InputPlayer::One => Sprite::CrabOrange,
            InputPlayer::Two => Sprite::CrabGreen,
        };

        Fighter {
            player,
            mode: FighterMode::Idle,
            animation: Animation::CrabIdle.instance(),
            input_queue: VecDeque::new(),

            sprite,
            sprite_frame_size: render_get_sprite_frame_size(sprite),

            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            direction: match player {
                InputPlayer::One => FighterDirection::Right,
                InputPlayer::Two => FighterDirection::Left
            },

            jumped_this_frame: false,
            has_double_jump: true,
            is_grounded: false,
            coyote_timer: 0,

            hitstun_timer: 0,
            damage: 0.0,
            has_hit: false
        }
    }

    pub fn update(&mut self, colliders: &Vec<Rect>) {
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

        // Reduce TTL on all inputs
        for input in self.input_queue.iter_mut() {
            input.ttl -= 1;
        }

        // Pop any inputs whose TTL is 0
        while !self.input_queue.is_empty() && self.input_queue.front().unwrap().ttl == 0 {
            self.input_queue.pop_front();
        }

        // Try to handle an input from the queue
        for index in 0..self.input_queue.len() {
            let input = self.input_queue.get(index).unwrap();
            if self.handle_input(input.typ) {
                // If we handled an input, clear the queue
                // self.input_queue.clear();
                self.input_queue.drain(0..(index + 1));
                break;
            }
        }

        // UPDATE

        match self.mode {
            FighterMode::Idle => {
                let di = self.get_directional_input();

                // Update direction
                if self.is_grounded && di != 0.0 {
                    self.direction = if di > 0.0 {
                        FighterDirection::Right
                    } else {
                        FighterDirection::Left
                    };
                }

                // Turn around on the spot when grounded
                if self.is_grounded &&
                    ((di == 1.0 && self.velocity.x < 0.0) ||
                    (di == -1.0 && self.velocity.x > 0.0))
                {
                    self.velocity.x = 0.0;
                }
            },
            FighterMode::Hitstun => {
                self.hitstun_timer -= 1;
                if self.hitstun_timer == 0 {
                    self.mode = FighterMode::Idle
                }
            },
            FighterMode::Neutral1 | FighterMode::Neutral2 | FighterMode::Neutral3 => {
                if self.animation.is_finished() {
                    self.mode = FighterMode::Idle;
                }
            }
        }

        // MOVE

        let was_grounded = self.is_grounded;
        self.update_velocity();
        self.is_grounded = false;
        self.move_x(colliders);
        self.move_y(colliders);

        // Coyote timer
        if !self.is_grounded && was_grounded && !self.jumped_this_frame {
            self.coyote_timer = FIGHTER_COYOTE_TIMER_DURATION;
        }
        if self.coyote_timer != 0 {
            self.coyote_timer -= 1;
        }

        // Reset jumps remaining
        self.jumped_this_frame = false;
        if self.is_grounded {
            self.has_double_jump = true;
        }
    }

    // QUEUE INPUT

    fn queue_input(&mut self, typ: FighterInputType) {
        if self.input_queue.len() == FIGHTER_INPUT_QUEUE_MAX_SIZE {
            self.input_queue.pop_front();
        }
        self.input_queue.push_back(FighterInput {
            typ,
            ttl: FIGHTER_INPUT_TTL
        });
    }

    // HANDLE INPUT

    fn handle_input(&mut self, input_type: FighterInputType) -> bool {
        match input_type {
            FighterInputType::Jump => self.handle_input_jump(),
            FighterInputType::Punch => self.handle_input_punch()
        }
    }

    fn handle_input_jump(&mut self) -> bool {
        if self.mode != FighterMode::Idle {
            return false;
        }

        if self.can_ground_jump() {
            // Begin jump
            self.jump();
            self.jumped_this_frame = true;
            self.coyote_timer = 0;

            return true;
        } else if self.has_double_jump {
            self.jump();
            self.has_double_jump = false;

            return true;
        }

        false
    }

    fn handle_input_punch(&mut self) -> bool {
        if self.is_grounded {
            if self.mode == FighterMode::Idle {
                self.set_attack_mode(FighterMode::Neutral1);
            } else if self.mode == FighterMode::Neutral1 && self.animation.is_on_recovery_frame() {
                self.set_attack_mode(FighterMode::Neutral2);
            } else if self.mode == FighterMode::Neutral2 && self.animation.is_on_recovery_frame() {
                self.set_attack_mode(FighterMode::Neutral3);
            }

            return true;
        }

        false
    }

    fn set_attack_mode(&mut self, mode: FighterMode) {
        self.mode = mode;
        self.has_hit = true;
        self.reset_animation();
    }

    // JUMP

    fn can_ground_jump(&self) -> bool {
        self.is_grounded || self.coyote_timer != 0
    }

    fn jump(&mut self) {
        self.mode = FighterMode::Idle;
        self.velocity.y = FIGHTER_JUMP_ACCELERATION;
        self.reset_animation();
    }

    fn get_directional_input(&self) -> f32 {
        let can_di = self.mode == FighterMode::Idle;
        if can_di && input_is_action_pressed(self.player, InputAction::Right) {
            1.0
        } else if can_di && input_is_action_pressed(self.player, InputAction::Left) {
            -1.0
        } else {
            0.0
        }
    }

    // ANIMATION

    fn reset_animation(&mut self) {
        self.animation = self.get_expected_animation().instance();
    }

    fn get_expected_animation(&self) -> Animation {
        match self.mode {
            FighterMode::Idle => {
                if !self.is_grounded {
                    return Animation::CrabFall
                }

                if self.velocity.x != 0.0 {
                    return Animation::CrabWalk
                }

                Animation::CrabIdle
            },
            FighterMode::Hitstun => Animation::CrabHurt,
            FighterMode::Neutral1 | FighterMode::Neutral2 => Animation::CrabPunch,
            FighterMode::Neutral3 => Animation::CrabPunch2
        }
    }

    // MOVE

    fn update_velocity(&mut self) {
        let di = self.get_directional_input();

        // Deceleration
        if di == 0.0 && self.velocity.x > 0.0 {
            self.velocity.x = (self.velocity.x - FIGHTER_WALK_DECELERATION).max(0.0);
        } else if di == 0.0 && self.velocity.x < 0.0 {
            self.velocity.x = (self.velocity.x + FIGHTER_WALK_DECELERATION).min(0.0);
        }

        // Walk acceleration
        if di == 1.0 && self.velocity.x < FIGHTER_WALK_SPEED {
            let x_acceleration = if self.is_grounded {
                FIGHTER_WALK_ACCELERATION
            } else {
                FIGHTER_AIR_ACCELERATION
            };
            self.velocity.x = (self.velocity.x + x_acceleration).min(FIGHTER_WALK_SPEED);
        } else if di == -1.0 && self.velocity.x > -FIGHTER_WALK_SPEED {
            self.velocity.x = (self.velocity.x - FIGHTER_WALK_ACCELERATION).max(-FIGHTER_WALK_SPEED);
        }
        if self.mode == FighterMode::Neutral1 || self.mode == FighterMode::Neutral2 || self.mode == FighterMode::Neutral3 {
            self.velocity.x = 0.0;
        }

        // Gravity
        self.velocity.y += FIGHTER_GRAVITY;
        if self.velocity.y > FIGHTER_FALL_SPEED {
            self.velocity.y = FIGHTER_FALL_SPEED;
        }
    }

    fn move_x(&mut self, colliders: &Vec<Rect>) {
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
    }

    fn move_y(&mut self, colliders: &Vec<Rect>) {
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
    }

    // COLLISION RESOLUTION

    pub fn handle_pushbox_collision(&mut self, collision: Vec2) {
        if self.is_grounded {
            self.position.x += collision.x * 0.5;
        }
    }

    // COLLIDERS

    fn get_rect(&self, offset: Vec2, size: Vec2) -> Rect {
        let offset = match self.direction {
            FighterDirection::Right => offset,
            FighterDirection::Left => Vec2::new(self.sprite_frame_size.x - size.x - offset.x, offset.y)
        };
        Rect {
            position: self.position + offset,
            size
        }
    }

    pub fn get_pushbox(&self) -> Rect {
        self.get_rect(Vec2::new(9.0, 5.0), Vec2::new(14.0, 11.0))
    }

    // Receives damage
    pub fn get_hurtbox(&self) -> Rect {
        self.get_rect(Vec2::new(9.0, 5.0), Vec2::new(14.0, 11.0))
    }

    // Deals damage
    pub fn get_hit_info(&self) -> Option<FighterHitInfo> {
        if !self.has_hit || !self.animation.is_on_hit_frame() {
            return None;
        }


        let knockbox_x_direction = (self.direction as i8) as f32;
        match self.mode {
            FighterMode::Neutral1 | FighterMode::Neutral2 => Some(FighterHitInfo {
                damage: 2.0,
                knockback_strength: 2.0,
                knockback_direction: Vec2::new(1.0 * knockbox_x_direction, -0.3).normalize(),
                hitbox: self.get_rect(Vec2::new(20.0, 3.0), Vec2::new(12.0, 7.0))
            }),
            FighterMode::Neutral3 => Some(FighterHitInfo {
                damage: 3.5,
                knockback_strength: 3.0,
                knockback_direction: Vec2::new(1.0 * knockbox_x_direction, -0.5).normalize(),
                hitbox: self.get_rect(Vec2::new(16.0, 5.0), Vec2::new(15.0, 8.0))
            }),
            _ => {
                assert!(false);
                None
            }
        }
    }

    // ON HIT

    pub fn handle_hit(&mut self, damage: f32, knockback_strength: f32, knockback_direction: Vec2) {
        self.damage += damage;
        let knockback_strength = 0.2 * (knockback_strength + ((self.damage / 10.0) + ((self.damage * damage) / 20.0)));
        self.velocity = knockback_strength * knockback_direction;
        self.mode = FighterMode::Hitstun;
        self.hitstun_timer = 5;
    }

    pub fn render(&self) {
        render_sprite(self.sprite, self.position, self.animation.h_frame, self.animation.v_frame, self.direction == FighterDirection::Left);
    }
}
