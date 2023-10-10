// use super::shared_defines::{ServerProcessType, ThisServerProcess};

// struct SecretInfo {
//     owner: ServerProcessType,
// }

// impl SecretInfo {
//     fn defer_load(&self) -> bool {
//         self.owner != ThisServerProcess::get()
//     }
// }

// const TOTP_SECRET_INFO: SecretInfo = SecretInfo {
//     owner: ServerProcessType::Authserver,
// };

// pub struct SecretMgr;

// SecretFlags::AUTHSERVER_DEFER_LOAD = 2^0 = 1;
// SecretFlags::WORLDSERVER_DEFER_LOAD = 2^16 = 65536;
// SecretFlags::SECRET_FLAG_DEFER_LOAD = 2^0 = 1;

// _flags = WORLDSERVER_DEFER_LOAD
// THIS_SERVER_PROCESS =     Authserver = 0 or Worldserver = 1,
// _flags >> (16*THIS_SERVER_PROCESS)
