use strum_macros::{EnumCount, EnumIter};
use strum::IntoEnumIterator;
use std::sync::OnceLock;

#[derive(EnumCount, EnumIter, Copy, Clone)]
pub enum Animation {
    CrabIdle,
    CrabWalk
}

struct AnimationFrame {
    h_frame: u32,
    v_frame: u32,
    duration: u32
}

struct AnimationData {
    frames: Vec<AnimationFrame>
}

pub struct AnimationInstance {
    animation: Animation,
    frame_index: usize,
    frame_timer: u32,
    pub h_frame: u32,
    pub v_frame: u32
}

impl Animation {
    pub fn instance(self) -> AnimationInstance {
        let animation_data: &'static AnimationData = self.get_data();
        AnimationInstance {
            animation: self,
            frame_index: 0,
            frame_timer: animation_data.frames[0].duration,
            h_frame: animation_data.frames[0].h_frame,
            v_frame: animation_data.frames[0].v_frame
        }
    }

    fn get_data(self) -> &'static AnimationData {
        let animation_data = ANIMATION_DATA.get().unwrap();
        &animation_data[self as usize]
    }
}

impl AnimationInstance {
    pub fn update(&mut self) {
        let animation_data: &'static AnimationData = self.animation.get_data();

        self.frame_timer -= 1;
        if self.frame_timer == 0 {
            self.frame_index = (self.frame_index + 1) % animation_data.frames.len();
            self.frame_timer = animation_data.frames[self.frame_index].duration;
            self.h_frame = animation_data.frames[self.frame_index].h_frame;
            self.v_frame = animation_data.frames[self.frame_index].v_frame;
        }
    }
}

static ANIMATION_DATA: OnceLock<Vec<AnimationData>> = OnceLock::new();

pub fn animation_init() {
    let mut animation_data: Vec<AnimationData> = Vec::new();
    for _animation in Animation::iter() {
        animation_data.push(AnimationData {
            frames: Vec::new()
        });
    }

    animation_data[Animation::CrabIdle as usize].frames = vec![
        AnimationFrame { h_frame: 0, v_frame: 0, duration: 16 },
        AnimationFrame { h_frame: 1, v_frame: 0, duration: 16 }
    ];
    animation_data[Animation::CrabWalk as usize].frames = vec![
        AnimationFrame { h_frame: 0, v_frame: 0, duration: 16 },
        AnimationFrame { h_frame: 2, v_frame: 0, duration: 16 },
        AnimationFrame { h_frame: 3, v_frame: 0, duration: 16 },
        AnimationFrame { h_frame: 4, v_frame: 0, duration: 16 }
    ];

    ANIMATION_DATA.get_or_init(|| animation_data);
}
