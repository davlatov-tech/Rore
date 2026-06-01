use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct TimeManager {
    last_update: Instant,
    pub dt: f32,          // Kadrlar oralig'idagi o'zgaruvchan vaqt
    pub elapsed: f32,     // Umumiy o'tgan vaqt
    pub accumulator: f32, // Fixed Timestep uchun vaqt yig'uvchi
    pub fixed_dt: f32,    // Qat'iy mantiqiy qadam (Masalan: 120Hz uchun 1/120)
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            dt: 0.0,
            elapsed: 0.0,
            accumulator: 0.0,
            fixed_dt: 1.0 / 120.0, // 120Hz garmoniya
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.dt = now.duration_since(self.last_update).as_secs_f32();

        if self.dt > 0.1 {
            self.dt = 1.0 / 60.0;
        }

        self.last_update = now;
        self.elapsed += self.dt;
    }

    pub fn add_accum(&mut self, dt: f32) {
        self.accumulator += dt;

        if self.accumulator > 0.5 {
            self.accumulator = 0.5;
        }
    }

    pub fn consume_fixed_step(&mut self) -> bool {
        if self.accumulator >= self.fixed_dt {
            self.accumulator -= self.fixed_dt;
            true
        } else {
            false
        }
    }
}
