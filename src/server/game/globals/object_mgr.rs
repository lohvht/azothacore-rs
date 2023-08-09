use std::sync::RwLock;

pub struct ObjectMgr {}

impl ObjectMgr {
    pub const fn new() -> ObjectMgr {
        ObjectMgr {}
    }

    pub fn get_script_id(&self, _name: &str) -> Result<i64, Box<dyn std::error::Error>> {
        todo!("NOT IMPL");
    }
}

pub static S_OBJECT_MGR: RwLock<ObjectMgr> = RwLock::new(ObjectMgr::new());
