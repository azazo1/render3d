use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct FpsCounter {
    count_window: Duration,
    tick_history: VecDeque<Instant>,
}

impl FpsCounter {
    const MIN_SAMPLES: usize = 3;
    pub fn new(count_window: Duration) -> Self {
        Self {
            count_window,
            tick_history: [].into(),
        }
    }

    /// 返回 fps
    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        self.tick_history.push_back(Instant::now());

        while self.tick_history.len() >= Self::MIN_SAMPLES
            && let Some(f) = self.tick_history.front()
            && *f < now - self.count_window
        {
            self.tick_history.pop_front();
        }

        if let Some(f) = self.tick_history.front() {
            self.tick_history.len() as f32 / (now - *f).as_secs_f32().max(f32::EPSILON)
        } else {
            0.0
        }
    }
}
