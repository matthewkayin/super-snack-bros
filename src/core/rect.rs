use glam::Vec2;

pub struct Rect {
    pub position: Vec2,
    pub size: Vec2
}

impl Rect {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Rect {
            position,
            size
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.x < other.position.x ||
            other.position.x + other.size.x < self.size.x ||
            self.position.y + self.size.y < other.size.y ||
            other.position.y + other.size.y < self.size.y)
    }
}
