use instant::{Duration, Instant};

pub struct FrameCounter {
    rate: Duration,
    last_ticks: u64,
    ticks: u64,
    last_time: Instant,
    time: Instant,
}
impl Default for FrameCounter {
    fn default() -> Self {
        FrameCounter {
            rate: Duration::from_secs(1),
            last_ticks: 0,
            ticks: 0,
            last_time: Instant::now(),
            time: Instant::now(),
        }
    }
}
impl FrameCounter {
    pub fn tick(&mut self) -> (Duration, Option<f64>) {
        self.ticks += 1;
        let now = Instant::now();
        let tick_dur = now.duration_since(self.time);
        self.time = now;

        let fps_dur = self.time.duration_since(self.last_time);
        let fps_opt = if fps_dur >= self.rate {
            self.last_time = self.time;
            let fps = (self.ticks - self.last_ticks) as f64 / fps_dur.as_secs_f64();
            self.last_ticks = self.ticks;
            Some(fps)
        } else {
            None
        };
        (tick_dur, fps_opt)
    }
}
