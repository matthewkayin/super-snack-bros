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

pub struct GameState {
    players: [Fighter; INPUT_PLAYER_COUNT],
    level_platforms: Vec<Rect>,
    parallax_x: f32
}

impl GameState {
    pub fn new() -> Self {
        // INIT LEVEL PLATFORMS

        let mut level_platforms = Vec::new();

        // Center platform
        let center_platform_width = TILE_SIZE_F32 * 16.0;
        let center_platform_height = TILE_SIZE_F32;
        let center_platform_x = (SCREEN_WIDTH - center_platform_width) / 2.0;
        let center_platform_y = SCREEN_HEIGHT - (TILE_SIZE_F32 * 2.0);
        level_platforms.push(Rect {
            position: Vec2::new(center_platform_x, center_platform_y),
            size: Vec2::new(center_platform_width, center_platform_height)
        });

        // Determine player positions
        let mut players = [Fighter::new(InputPlayer::One), Fighter::new(InputPlayer::Two)];
        for player_index in 0..2 {
            let sprite_frame_size = render_get_sprite_frame_size(players[player_index].sprite);
            let player_spawn_offset = TILE_SIZE_F32 * 2.0;
            let player_center_x = match player_index {
                0 => center_platform_x + player_spawn_offset,
                1 => center_platform_x + center_platform_width - player_spawn_offset,
                _ => { assert!(false); 0.0 }
            };
            let player_x = player_center_x - (sprite_frame_size.x / 2.0);
            let player_y = center_platform_y - sprite_frame_size.y - 100.0;

            players[player_index].position = Vec2::new(player_x, player_y);
        }

        GameState {
            players,
            level_platforms,
            parallax_x: 0.0
        }
    }

    pub fn update(&mut self) {
        for player in self.players.iter_mut() {
            player.update(&self.level_platforms);
        }

        let pushbox_collision = self.players[0].get_pushbox().get_collision(&self.players[1].get_pushbox());
        self.players[0].handle_pushbox_collision(pushbox_collision);
        self.players[1].handle_pushbox_collision(-pushbox_collision);

        self.parallax_x = (self.parallax_x + PARALLAX_SPEED) % PARALLAX_WIDTH;
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

        let rect_color_green = "#00ff00ff";

        for player in self.players.iter() {
            player.render();
            let pushbox = player.get_pushbox();
            render_draw_rect(&rect_color_green, pushbox.position, pushbox.size);
        }

        for collider in self.level_platforms.iter() {
            render_draw_rect(&rect_color_green, collider.position, collider.size);
        }
    }

    fn render_platform(platform: &Rect) {
        let platform_size_x = platform.size.x as u32;
        let platform_size_y = (SCREEN_HEIGHT - platform.position.y) as u32;

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
}
