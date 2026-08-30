use glam::Vec2;

pub struct Rect {
    pub position: Vec2,
    pub size: Vec2
}

impl Rect {
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.x < other.position.x ||
            other.position.x + other.size.x < self.size.x ||
            self.position.y + self.size.y < other.size.y ||
            other.position.y + other.size.y < self.size.y)
    }

    pub fn get_collision_x(&self, other: &Rect) -> f32 {
        let mut collision_x = 0.0;

        // First, check that we are aligned on the y axis
        let vertically_overlapping = !(
            self.position.y + self.size.y < other.position.y ||
            other.position.y + other.size.y < self.position.y);

        if vertically_overlapping {
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

    pub fn get_collision_y(&self, other: &Rect) -> f32 {
        let mut collision_y = 0.0;

        // First, check that we are aligned on the x axis
        let horizontally_overlapping = !(
            self.position.x + self.size.x < other.position.x ||
            other.position.x + other.size.x < self.position.x);

        if horizontally_overlapping {
            if self.position.y + self.size.y > other.position.y &&
                self.position.y + self.size.y < other.position.y + other.size.y
            {
                collision_y = other.position.y - (self.position.y + self.size.y);
            } else if self.position.y > other.position.y &&
                self.position.y < other.position.y + other.size.y
            {
                collision_y = (other.position.y + other.size.y) - self.position.y;
            }
        }

        collision_y
    }
}
