use std::time::{Instant, Duration};

use crate::asset_manager::AssetHandle;

use super::Component;


pub struct AnimationState {
    pub start_time: Instant,
    pub current_time: Instant,
    pub index: usize,
}

impl AnimationState {
    pub fn new(index: usize) -> Self {
        let start_time = Instant::now();
        Self {
            start_time,
            current_time: start_time,
            index,
        }
    }
    pub fn advance(&mut self, delta: f32) {
        let duration = Duration::from_secs_f32(delta);
        self.current_time += duration;
    }
}

pub struct Model {
    pub asset_handle: AssetHandle,
    pub animation: Option<AnimationState>,
}

impl Component for Model {

}
