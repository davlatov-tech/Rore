use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct TimeManager {
    last_update: Instant,
    pub dt: f32,      // Kadrlar oralig'idagi vaqt (Delta Time)
    pub elapsed: f32, // Umumiy vaqt
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            dt: 0.0,
            elapsed: 0.0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.elapsed += self.dt;
    }
}