use crate::game::fighter::*;
use crate::core::input::*;
use crate::core::render::*;
use crate::game::rect::*;
use crate::constants::*;
use glam::Vec2;

const PARALLAX_SPEED: f32 = 0.2;
const PARALLAX_WIDTH: f32 = 256.0;

const TILE_SIZE_U32: u32 = 16;
const TILE_SIZE_F32: f32 = TILE_SIZE_U32 as f32;

const DEBUG_MODE_NONE: u32 = 0;
const DEBUG_MODE_PUSHBOXES: u32 = 1;
const DEBUG_MODE_HITBOXES: u32 = 2;
const DEBUG_MODE_COUNT: u32 = 3;

pub struct GameState {
    players: [Fighter; INPUT_PLAYER_COUNT],
    level_platforms: Vec<Rect>,
    parallax_x: f32,

    debug_mode: u32
}

impl GameState {
    pub fn new() -> Self {
        // INIT LEVEL PLATFORMS

        // Center platform
        let center_platform_width = TILE_SIZE_F32 * 16.0;
        let center_platform_height = TILE_SIZE_F32 * 2.0;
        let center_platform_x = (SCREEN_WIDTH - center_platform_width) / 2.0;
        let center_platform_y = SCREEN_HEIGHT - (TILE_SIZE_F32 * 2.0);

        // Side platforms
        let side_platform_width = TILE_SIZE_F32 * 5.0;
        let side_platform_height = TILE_SIZE_F32;
        let side_platform_margin = TILE_SIZE_F32 * 0.5;
        let side_platform_y = center_platform_y - (TILE_SIZE_F32 * 4.0);
        let left_platform_x = center_platform_x + side_platform_margin;
        let right_platform_x = center_platform_x + center_platform_width - side_platform_margin - side_platform_width;

        // Top platform
        let top_platform_x = (SCREEN_WIDTH / 2.0) - (side_platform_width / 2.0);
        let top_platform_y = center_platform_y - (TILE_SIZE_F32 * 8.0);

        let level_platforms = vec![
            // Top
            Rect {
                position: Vec2::new(top_platform_x, top_platform_y),
                size: Vec2::new(side_platform_width, side_platform_height)
            },
            // Left
            Rect {
                position: Vec2::new(left_platform_x, side_platform_y),
                size: Vec2::new(side_platform_width, side_platform_height)
            },
            // Right
            Rect {
                position: Vec2::new(right_platform_x, side_platform_y),
                size: Vec2::new(side_platform_width, side_platform_height)
            },
            // Center
            Rect {
                position: Vec2::new(center_platform_x, center_platform_y),
                size: Vec2::new(center_platform_width, center_platform_height)
            }
        ];

        // Determine player positions
        let mut players = [Fighter::new(InputPlayer::One), Fighter::new(InputPlayer::Two)];
        for player_index in 0..INPUT_PLAYER_COUNT {
            let sprite_frame_size = render_get_sprite_frame_size(players[player_index].sprite);
            let player_spawn_offset = TILE_SIZE_F32 * 2.0;
            let player_center_x = match player_index {
                0 => center_platform_x + player_spawn_offset,
                1 => center_platform_x + center_platform_width - player_spawn_offset,
                _ => { assert!(false); 0.0 }
            };
            let player_x = player_center_x - (sprite_frame_size.x / 2.0);
            let player_y = center_platform_y - sprite_frame_size.y;

            players[player_index].position = Vec2::new(player_x, player_y);
        }

        GameState {
            players,
            level_platforms,
            parallax_x: 0.0,

            debug_mode: DEBUG_MODE_NONE
        }
    }

    pub fn update(&mut self) {
        // Update players
        for player in self.players.iter_mut() {
            player.update(&self.level_platforms);
        }

        // Handle player-to-player collision
        let pushbox_collision = self.players[0].get_pushbox().get_collision_x(&self.players[1].get_pushbox());
        self.players[0].handle_pushbox_collision(Vec2::new(pushbox_collision, 0.0));
        self.players[1].handle_pushbox_collision(Vec2::new(-pushbox_collision, 0.0));

        // Handle player hits
        let player_hitbox_opts: [Option<Rect>; INPUT_PLAYER_COUNT] =
            self.players.iter()
            .map(|player| player.get_hitbox())
            .collect::<Vec<_>>().try_into().unwrap();
        let player_hurtboxes: [Rect; INPUT_PLAYER_COUNT] =
            self.players.iter()
            .map(|player| player.get_hurtbox())
            .collect::<Vec<_>>().try_into().unwrap();

        for index in 0..INPUT_PLAYER_COUNT {
            let opp_index = if index == 0 { 1 } else { 0 };
            if let Some(hitbox) = &player_hitbox_opts[index] {
                let intersects_opp_hitbox = match &player_hitbox_opts[opp_index] {
                    Some(opp_hitbox) => hitbox.intersects(&opp_hitbox),
                    None => false
                };
                if !intersects_opp_hitbox && hitbox.intersects(&player_hurtboxes[opp_index]) {
                    self.players[opp_index].handle_hit();
                }
            }
        }

        // Update parallax
        self.parallax_x = (self.parallax_x + PARALLAX_SPEED) % PARALLAX_WIDTH;

        // Toggle debug modes
        if input_is_action_just_pressed(InputPlayer::One, InputAction::Start) {
            self.debug_mode = (self.debug_mode + 1) % DEBUG_MODE_COUNT;
        }
    }

    pub fn render(&self) {
        // Background
        let background_color = "#309395";
        render_fill_rect(&background_color, Vec2::new(0.0, 0.0), Vec2::new(SCREEN_WIDTH, SCREEN_HEIGHT));

        // Parallax
        let mut parallax_x = (-self.parallax_x).floor();
        while parallax_x < SCREEN_WIDTH {
            render_sprite(Sprite::Parallax, Vec2::new(parallax_x, SCREEN_HEIGHT - PARALLAX_WIDTH), 0, 0, false);
            parallax_x += PARALLAX_WIDTH;
        }

        // Platforms
        for collider in self.level_platforms.iter() {
            GameState::render_platform(&collider);
        }

        // Player
        for player in self.players.iter() {
            player.render();
        }

        // Debug - Pushboxes
        if self.debug_mode == DEBUG_MODE_PUSHBOXES {
            let rect_color_green = "#00ff00ff";

            // Level colliders
            for collider in self.level_platforms.iter() {
                render_draw_rect(&rect_color_green, collider.position, collider.size);
            }

            // Player pushboxes
            for player in self.players.iter() {
                let pushbox = player.get_pushbox();
                render_draw_rect(&rect_color_green, pushbox.position, pushbox.size);
            }
        }

        // Debug - Hitboxes
        if self.debug_mode == DEBUG_MODE_HITBOXES {
            let rect_color_red = "#ff0000ff";
            let rect_color_blue = "#0000ffff";

            // Player hurtboxes
            for player in self.players.iter() {
                let hurtbox = player.get_hurtbox();
                render_draw_rect(&rect_color_blue, hurtbox.position, hurtbox.size);
            }

            // Player hitboxes
            for player in self.players.iter() {
                let hitbox = player.get_hitbox();
                match hitbox {
                    Some(collider) => render_draw_rect(&rect_color_red, collider.position, collider.size),
                    None => ()
                }
            }
        }

        // Health clusters
        self.render_health_cluster(InputPlayer::One);
        self.render_health_cluster(InputPlayer::Two);

        // Debug UI
        let text_position = Vec2::new(2.0, 10.0);
        match self.debug_mode {
            DEBUG_MODE_NONE => render_text("Debug: None", text_position),
            DEBUG_MODE_PUSHBOXES => render_text("Debug: Pushboxes", text_position),
            DEBUG_MODE_HITBOXES => render_text("Debug: Hitboxes", text_position),
            _ => ()
        }
    }

    fn render_platform(platform: &Rect) {
        let platform_size_x = platform.size.x as u32;
        let platform_size_y = platform.size.y as u32;

        assert!(platform_size_x % TILE_SIZE_U32 == 0);
        assert!(platform_size_y % TILE_SIZE_U32 == 0);

        let tile_width = platform_size_x / TILE_SIZE_U32;
        let tile_height = platform_size_y / TILE_SIZE_U32;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let h_frame = match x {
                    0 => 0,
                    _ if x == tile_width - 1 => 2,
                    _ => 1
                };
                let v_frame = match y {
                    0 => 1,
                    _ => 2
                };

                render_sprite(Sprite::Tileset, platform.position + Vec2::new((x * TILE_SIZE_U32) as f32, (y * TILE_SIZE_U32) as f32), h_frame, v_frame, false);
            }
        }
    }

    fn render_health_cluster(&self, player: InputPlayer) {
        let player_index = player as usize;

        // Render frame
        let frame_y = 237.0;
        let frame_x = match player {
            InputPlayer::One => 64.0,
            InputPlayer::Two => 202.0
        };
        let frame_position = Vec2::new(frame_x, frame_y);
        render_sprite(Sprite::HealthFrame, frame_position, player_index as u32, 0, false);

        // Render player sprite
        let sprite_position = frame_position + Vec2::new(-4.0, 4.0);
        render_sprite(self.players[player_index].sprite, sprite_position, 0, 0, false);

        // Determine font color
        let font_color = match self.players[player_index].damage {
            66.6.. => BitmapFontColor::Red,
            33.3.. => BitmapFontColor::Yellow,
            _ => BitmapFontColor::White
        };

        // Render damage text
        let damage_int_part = self.players[player_index].damage.floor();
        let damage_int_part_str = format!("{}", damage_int_part as u32);
        let text_position = frame_position + Vec2::new(28.0, 7.0);
        let text_width = render_bitmap_text(&damage_int_part_str, BitmapFont::Numbers28, font_color, text_position);

        let damage_frac_part = ((self.players[player_index].damage - damage_int_part) * 10.0).floor();
        let damage_frag_part_str = format!(".{}%", damage_frac_part as u32);
        let text_position = text_position + Vec2::new(text_width, 7.0);
        render_bitmap_text(&damage_frag_part_str, BitmapFont::Numbers16, font_color, text_position);
    }
}
