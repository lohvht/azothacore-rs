use tokio::sync::RwLock as AsyncRwLock;

/// max allowed by client name length
pub const MAX_PLAYER_NAME: u8 = 12;
/// max server internal player name length (> MAX_PLAYER_NAME for support declined names)
pub const MAX_INTERNAL_PLAYER_NAME: u8 = 15;
/// max allowed by client name length
pub const MAX_PET_NAME: u8 = 12;
/// max allowed by client name length
pub const MAX_CHARTER_NAME: u8 = 24;

pub struct ObjectMgr {}

impl ObjectMgr {
    pub const fn new() -> ObjectMgr {
        ObjectMgr {}
    }

    pub fn get_script_id(&self, _name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        todo!("NOT IMPL");
    }
}

impl Default for ObjectMgr {
    fn default() -> Self {
        Self::new()
    }
}

pub static OBJECT_MGR: AsyncRwLock<ObjectMgr> = AsyncRwLock::const_new(ObjectMgr::new());
