use super::WorldError;

pub trait WorldTrait {
    fn is_stopped(&self) -> bool;
    async fn load_db_version(&mut self) -> Result<(), WorldError>;
    fn get_db_version(&self) -> &String;
    fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError>;
}
