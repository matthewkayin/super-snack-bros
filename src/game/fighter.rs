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
    Hitstun,
    PunchGround1,
    PunchGround2
}

#[repr(i8)]
#[derive(PartialEq, Eq)]
enum FighterDirection {
    Right = 1,
    Left = -1
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
    mode: FighterMode,
    animation: AnimationInstance,
    input_queue: VecDeque<FighterInput>,

    pub sprite: Sprite,
    sprite_frame_size: Vec2,

    pub position: Vec2,
    pub velocity: Vec2,
    direction: FighterDirection,

    has_double_jump: bool,
    is_grounded: bool,
    coyote_timer: u32,
    jump_timer: u32,

    hitstun_timer: u32,
    damage: f32
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

            has_double_jump: false,
            is_grounded: false,
            coyote_timer: 0,
            jump_timer: 0,

            hitstun_timer: 0,
            damage: 0.0
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
            FighterMode::Hitstun => {
                self.hitstun_timer -= 1;
                if self.hitstun_timer == 0 {
                    self.mode = FighterMode::Idle
                }
            },
            FighterMode::PunchGround1 | FighterMode::PunchGround2 => {
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

    // JUMP

    fn can_ground_jump(&self) -> bool {
        (self.is_grounded || self.coyote_timer != 0) &&
        self.mode == FighterMode::Idle
    }

    fn jump(&mut self) {
        self.velocity.y = if self.can_ground_jump() && !input_is_action_pressed(self.player, InputAction::Up) {
            FIGHTER_JUMP_SHORT_HOP_ACCELERATION
        } else {
            FIGHTER_JUMP_ACCELERATION
        };
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
            FighterMode::JumpSquat => Animation::CrabJump,
            FighterMode::Hitstun => Animation::CrabHurt,
            FighterMode::PunchGround1 | FighterMode::PunchGround2 => Animation::CrabPunch
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
            self.velocity.x = (self.velocity.x + FIGHTER_WALK_ACCELERATION).min(FIGHTER_WALK_SPEED);
        } else if di == -1.0 && self.velocity.x > -FIGHTER_WALK_SPEED {
            self.velocity.x = (self.velocity.x - FIGHTER_WALK_ACCELERATION).max(-FIGHTER_WALK_SPEED);
        }
        if self.mode == FighterMode::PunchGround1 || self.mode == FighterMode::PunchGround2 {
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
    pub fn get_hitbox(&self) -> Option<Rect> {
        match self.mode {
            FighterMode::PunchGround1 | FighterMode::PunchGround2 => Some(
                self.get_rect(Vec2::new(20.0, 3.0), Vec2::new(12.0, 7.0))
            ),
            _ => None
        }
    }

    // ON HIT

    pub fn handle_hit(&mut self) {
        self.velocity = Vec2::ZERO;
        self.mode = FighterMode::Hitstun;
        self.hitstun_timer = 5;
    }

    pub fn render(&self) {
        render_sprite(self.sprite, self.position, self.animation.h_frame, self.animation.v_frame, self.direction == FighterDirection::Left);
    }
}
