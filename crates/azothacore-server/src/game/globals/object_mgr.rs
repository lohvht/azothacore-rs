use std::sync::RwLock;

pub struct ObjectMgr {}

impl ObjectMgr {
    pub const fn new() -> ObjectMgr {
        ObjectMgr {}
    }

    pub fn get_script_id(&self, _name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        todo!("NOT IMPL");
    }
}

pub static OBJECT_MGR: RwLock<ObjectMgr> = RwLock::new(ObjectMgr::new());
