use crate::game::fighter::*;
use crate::core::input::*;
use crate::core::render::*;
use crate::game::collider::*;
use crate::constants::*;
use glam::Vec2;

pub struct GameState {
    players: [Fighter; INPUT_PLAYER_COUNT],
    level_colliders: Vec<Collider>
}

impl GameState {
    pub fn new() -> Self {
        let players = [Fighter::new(InputPlayer::One), Fighter::new(InputPlayer::Two)];

        let mut level_colliders = Vec::new();
        level_colliders.push(Collider::new(Vec2::new(0.0, SCREEN_HEIGHT), Vec2::new(SCREEN_WIDTH, 16.0)));

        GameState {
            players,
            level_colliders
        }
    }

    pub fn update(&mut self) {
        for player in self.players.iter_mut() {
            player.update();
        }

        let pushbox_collision = self.players[0].get_pushbox().get_collision(&self.players[1].get_pushbox());
        self.players[0].handle_pushbox_collision(pushbox_collision);
        self.players[1].handle_pushbox_collision(-pushbox_collision);

        for player in self.players.iter_mut() {
            player.handle_static_collisions(&self.level_colliders);
        }
    }

    pub fn render(&self) {
        let rect_color_green = "#00ff00ff";

        for player in self.players.iter() {
            player.render();
            render_rect(&rect_color_green, player.get_pushbox().position(), player.get_pushbox().size());
        }

        for collider in self.level_colliders.iter() {
            render_rect(&rect_color_green, collider.position(), collider.size());
        }
    }
}
