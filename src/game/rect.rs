use glam::Vec2;

pub struct Rect {
    pub position: Vec2,
    pub size: Vec2
}

impl Rect {
    pub fn intersects_horizontally(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.x <= other.position.x ||
            self.position.x >= other.position.x + other.size.x)
    }

    pub fn intersects_vertically(&self, other: &Rect) -> bool {
        !(self.position.y + self.size.y <= other.position.y ||
            self.position.y >= other.position.y + other.size.y)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.intersects_horizontally(other) && self.intersects_vertically(other)
    }

    pub fn get_collision_x(&self, other: &Rect) -> f32 {
        let mut collision_x = 0.0;

        if self.intersects_vertically(other) {
            if self.position.x + self.size.x > other.position.x &&
                self.position.x + self.size.x < other.position.x + other.size.x
            {
                collision_x = other.position.x - (self.position.x + self.size.x);
            } else if  self.position.x > other.position.x &&
                self.position.x < other.position.x + other.size.x
            {
                collision_x = (other.position.x + other.size.x) - self.position.x;
            }
        }

        collision_x
    }
}
