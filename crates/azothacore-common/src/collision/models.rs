pub mod game_object_model;
pub mod model_instance;
pub mod world_model;

flagset::flags! {
    pub enum ModelIgnoreFlags : u32
    {
        Nothing = 0x00,
        M2      = 0x01
    }
}
