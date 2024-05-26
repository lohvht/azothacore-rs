use std::{ops, sync::RwLock};

#[derive(Clone, Copy)]
pub struct MoveSpeed {
    walk:        f32,
    run:         f32,
    run_back:    f32,
    swim:        f32,
    swim_back:   f32,
    turn_rate:   f32,
    flight:      f32,
    flight_back: f32,
    pitch_rate:  f32,
}

impl MoveSpeed {
    pub const fn new() -> Self {
        Self {
            walk: 2.5,
            run: 7.0,
            run_back: 4.5,
            swim: 4.722222,
            swim_back: 2.5,
            turn_rate: 3.141594,
            flight: 7.0,
            flight_back: 4.5,
            #[expect(clippy::approx_constant)]
            pitch_rate: 3.14,
        }
    }
}

impl Default for MoveSpeed {
    fn default() -> Self {
        Self::new()
    }
}

impl ops::Mul<f32> for MoveSpeed {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            walk:        self.walk * rhs,
            run:         self.run * rhs,
            run_back:    self.run_back * rhs,
            swim:        self.swim * rhs,
            swim_back:   self.swim_back * rhs,
            turn_rate:   self.turn_rate * rhs,
            flight:      self.flight * rhs,
            flight_back: self.flight_back * rhs,
            pitch_rate:  self.pitch_rate * rhs,
        }
    }
}

pub static BASE_MOVE_SPEED: MoveSpeed = MoveSpeed::new();
pub static PLAYER_BASE_MOVE_SPEED: RwLock<MoveSpeed> = RwLock::new(MoveSpeed::new());
