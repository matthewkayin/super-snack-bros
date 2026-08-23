struct AnimationFrame {
    h_frame: u32,
    v_frame: u32,
    duration: u32
}

pub struct Animation {
    frames: &'static [AnimationFrame]
}

pub struct AnimationInstance {
    animation: &'static Animation,
    frame_index: usize,
    frame_timer: u32,
    pub h_frame: u32,
    pub v_frame: u32
}

impl Animation {
    pub fn instance(&'static self) -> AnimationInstance {
        AnimationInstance {
            animation: &self,
            frame_index: 0,
            frame_timer: self.frames[0].duration,
            h_frame: self.frames[0].h_frame,
            v_frame: self.frames[0].v_frame
        }
    }
}

impl AnimationInstance {
    pub fn update(&mut self) {
        self.frame_timer -= 1;
        if self.frame_timer == 0 {
            self.frame_index = (self.frame_index + 1) % self.animation.frames.len();
            self.frame_timer = self.animation.frames[self.frame_index].duration;
            self.h_frame = self.animation.frames[self.frame_index].h_frame;
            self.v_frame = self.animation.frames[self.frame_index].v_frame;
        }
    }
}

pub const ANIMATION_CRAB_IDLE: Animation = Animation {
    frames: &[
        AnimationFrame { h_frame: 0, v_frame: 0, duration: 16 },
        AnimationFrame { h_frame: 1, v_frame: 0, duration: 16 }
    ]
};
