use std::future::Future;

use super::WorldError;

pub trait WorldTrait {
    fn is_stopped(&self) -> bool;
    fn load_db_version(&mut self) -> impl Future<Output = Result<(), WorldError>> + Send;
    fn set_initial_world_settings(&mut self) -> impl Future<Output = Result<(), WorldError>> + Send;
    fn get_db_version(&self) -> &String;
    fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError>;
}
