use std::sync::OnceLock;

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, ToPrimitive, FromPrimitive, Deserialize, Serialize, PartialEq)]
pub enum ServerProcessType {
    Authserver = 0,
    Worldserver = 1,
}

pub struct ThisServerProcess;

impl ThisServerProcess {
    pub fn get() -> ServerProcessType {
        THIS_SERVER_PROCESS.get().expect("Server process not set").clone()
    }

    pub fn set(e: ServerProcessType) {
        THIS_SERVER_PROCESS.set(e).expect("Server process already set");
    }
}

static THIS_SERVER_PROCESS: OnceLock<ServerProcessType> = OnceLock::new();
