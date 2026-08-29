use glam::Vec2;

pub struct Collider {
    left_edge: f32,
    right_edge: f32,
    top_edge: f32,
    bottom_edge: f32
}

impl Collider {
    pub fn new(position: Vec2, size: Vec2) -> Collider {
        Collider {
            left_edge: position.x,
            right_edge: position.x + size.x,
            top_edge: position.y,
            bottom_edge: position.y + size.y
        }
    }

    pub fn position(&self) -> Vec2 {
        Vec2::new(self.left_edge, self.top_edge)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.right_edge - self.left_edge, self.bottom_edge - self.top_edge)
    }

    pub fn get_collision(&self, other: &Collider) -> Vec2 {
        let mut collision = Vec2::new(0.0, 0.0);

        let left_edge_aligns = self.left_edge >= other.left_edge && self.left_edge <= other.right_edge;
        let right_edge_aligns = self.right_edge >= other.left_edge && self.right_edge <= other.right_edge;
        let top_edge_aligns = self.top_edge >= other.top_edge && self.top_edge <= other.bottom_edge;
        let bottom_edge_aligns = self.bottom_edge >= other.top_edge && self.bottom_edge <= other.bottom_edge;

        // Horizontal
        if top_edge_aligns || bottom_edge_aligns {
            let left_edge_itersects = self.left_edge > other.left_edge && self.left_edge < other.right_edge;
            let right_edge_itersects = self.right_edge > other.left_edge && self.right_edge < other.right_edge;
            if right_edge_itersects && !left_edge_itersects {
                collision.x = other.left_edge - self.right_edge;
            } else if left_edge_itersects && !right_edge_itersects {
                collision.x = other.right_edge - self.left_edge;
            }
        }

        // Vertical
        if left_edge_aligns || right_edge_aligns {
            let top_edge_itersects = self.top_edge > other.top_edge && self.top_edge < other.bottom_edge;
            let bottom_edge_itersects = self.bottom_edge > other.top_edge && self.bottom_edge < other.bottom_edge;
            if bottom_edge_itersects && !top_edge_itersects {
                collision.y = other.top_edge - self.bottom_edge;
            } else if top_edge_itersects && !bottom_edge_itersects {
                collision.y = other.bottom_edge - self.top_edge;
            }
        }

        collision
    }
}
