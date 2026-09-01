use strum_macros::{EnumCount, EnumIter};
use strum::IntoEnumIterator;
use std::sync::OnceLock;

const ANIMATION_LOOPS_INDEFINITELY: u32 = u32::MAX;

#[derive(Debug, EnumCount, EnumIter, Copy, Clone, Eq, PartialEq)]
pub enum Animation {
    CrabIdle,
    CrabWalk,
    CrabJump,
    CrabFall,
    CrabHurt,
    CrabPunch,
    CrabPunch2,
    CrabSideSmash
}

struct AnimationFrame {
    h_frame: u32,
    v_frame: u32,
    duration: u32
}

struct AnimationData {
    loops: u32,
    hit_range: Option<(usize, usize)>,
    frames: Vec<AnimationFrame>
}

#[derive(Debug)]
pub struct AnimationInstance {
    pub name: Animation,
    frame_index: usize,
    frame_timer: u32,
    loops_remaining: u32,
    pub h_frame: u32,
    pub v_frame: u32
}

impl Animation {
    pub fn instance(self) -> AnimationInstance {
        let animation_data: &'static AnimationData = self.get_data();
        AnimationInstance {
            name: self,
            frame_index: 0,
            frame_timer: animation_data.frames[0].duration,
            loops_remaining: animation_data.loops,
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
        let animation_data: &'static AnimationData = self.name.get_data();

        self.frame_timer -= 1;
        if self.frame_timer == 0 {
            // Increase frame
            self.frame_index += 1;

            // If we've reached the last frame...
            if self.frame_index == animation_data.frames.len() {
                // Loop back around
                self.frame_index = 0;

                // And (optionally) decrement the loop counter
                if self.loops_remaining != ANIMATION_LOOPS_INDEFINITELY {
                    self.loops_remaining -= 1
                }
            }

            // If we're done looping, the animation is finished, so return
            if self.is_finished() {
                return;
            }

            // Otherwise, setup the next animation frame
            self.frame_timer = animation_data.frames[self.frame_index].duration;
            self.h_frame = animation_data.frames[self.frame_index].h_frame;
            self.v_frame = animation_data.frames[self.frame_index].v_frame;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.loops_remaining == 0
    }

    pub fn is_on_hit_frame(&self) -> bool {
        let animation_data: &'static AnimationData = self.name.get_data();
        match animation_data.hit_range {
            Some((hit_start, hit_end)) => self.frame_index >= hit_start && self.frame_index < hit_end,
            None => false
        }
    }

    pub fn is_on_recovery_frame(&self) -> bool {
        let animation_data: &'static AnimationData = self.name.get_data();
        match animation_data.hit_range {
            Some((_, hit_end)) => self.frame_index >= hit_end,
            None => false
        }
    }
}

static ANIMATION_DATA: OnceLock<Vec<AnimationData>> = OnceLock::new();

pub fn animation_init() {
    let mut animation_data: Vec<AnimationData> = Vec::new();
    for _animation in Animation::iter() {
        animation_data.push(AnimationData {
            frames: Vec::new(),
            hit_range: None,
            loops: 0
        });
    }

    // Crab Idle
    animation_data[Animation::CrabIdle as usize] = AnimationData {
        loops: ANIMATION_LOOPS_INDEFINITELY,
        hit_range: None,
        frames: vec![
            AnimationFrame { h_frame: 0, v_frame: 0, duration: 10 },
            AnimationFrame { h_frame: 1, v_frame: 0, duration: 10 }
        ]
    };

    // Crab Walk Forward
    animation_data[Animation::CrabWalk as usize] = AnimationData {
        loops: ANIMATION_LOOPS_INDEFINITELY,
        hit_range: None,
        frames: vec![
            AnimationFrame { h_frame: 0, v_frame: 0, duration: 8 },
            AnimationFrame { h_frame: 2, v_frame: 0, duration: 8 },
            AnimationFrame { h_frame: 3, v_frame: 0, duration: 8 },
            AnimationFrame { h_frame: 4, v_frame: 0, duration: 8 }
        ]
    };

    // Crab Jump
    animation_data[Animation::CrabJump as usize] = AnimationData {
        loops: ANIMATION_LOOPS_INDEFINITELY,
        hit_range: None,
        frames: vec![
            AnimationFrame { h_frame: 5, v_frame: 0, duration: 8 }
        ]
    };

    // Crab Fall
    animation_data[Animation::CrabFall as usize] = AnimationData {
        loops: ANIMATION_LOOPS_INDEFINITELY,
        hit_range: None,
        frames: vec![
            AnimationFrame { h_frame: 6, v_frame: 0, duration: 8 }
        ]
    };

    // Crab Hurt
    animation_data[Animation::CrabHurt as usize] = AnimationData {
        loops: ANIMATION_LOOPS_INDEFINITELY,
        hit_range: None,
        frames: vec![
            AnimationFrame { h_frame: 7, v_frame: 0, duration: 8 }
        ]
    };

    // Punch
    animation_data[Animation::CrabPunch as usize] = AnimationData {
        loops: 1,
        hit_range: Some((1, 2)),
        frames: vec![
            AnimationFrame { h_frame: 0, v_frame: 1, duration: 4 },
            AnimationFrame { h_frame: 1, v_frame: 1, duration: 4 },
            AnimationFrame { h_frame: 2, v_frame: 1, duration: 4 }
        ]
    };

    // Punch 2
    animation_data[Animation::CrabPunch2 as usize] = AnimationData {
        loops: 1,
        hit_range: Some((2, 3)),
        frames: vec![
            AnimationFrame { h_frame: 3, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 4, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 5, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 6, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 4, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 3, v_frame: 1, duration: 4  },
        ]
    };

    // Side smash
    animation_data[Animation::CrabSideSmash as usize] = AnimationData {
        loops: 1,
        hit_range: Some((2, 3)),
        frames: vec![
            AnimationFrame { h_frame: 7, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 0, v_frame: 2, duration: 4  },
            AnimationFrame { h_frame: 1, v_frame: 2, duration: 4  },
            AnimationFrame { h_frame: 2, v_frame: 2, duration: 4  },
            AnimationFrame { h_frame: 4, v_frame: 1, duration: 4  },
            AnimationFrame { h_frame: 3, v_frame: 1, duration: 4  },
        ]
    };

    ANIMATION_DATA.get_or_init(|| animation_data);
}
