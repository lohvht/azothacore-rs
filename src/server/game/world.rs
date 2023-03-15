use parking_lot::RwLock;
use tracing::info;

#[derive(Debug, Clone)]
pub enum WorldError {
    StopFailed,
    AlreadyStopped,
}

impl std::error::Error for WorldError {}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct World {
    exit_code: Option<i32>,
}

impl World {
    const fn new() -> World {
        World { exit_code: None }
    }

    pub fn is_stopped(&self) -> bool {
        self.exit_code.is_some()
    }

    pub fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError> {
        info!("Turning world flag to stopped");
        if self.is_stopped() {
            return Err(WorldError::AlreadyStopped);
        }
        self.exit_code = Some(exit_code);
        Ok(exit_code)
    }
}

pub static S_WORLD: RwLock<World> = RwLock::new(World::new());
