use parking_lot::RwLock;

pub enum ServerProcessType {
    Authserver,
    Worldserver,
}

pub static THIS_SERVER_PROCESS: RwLock<Option<ServerProcessType>> = RwLock::new(None);
