use bevy::{ecs::system::SystemId, prelude::*};

use crate::{
    game::scripting::script_object::{IntoScriptObject, Script, ScriptObjectTrait},
    input_script_non,
};

pub trait AccountScript: Script {
    /// AccountScript::OnAccountLogin in TC/AC
    /// Called when an account logged in successfully
    fn on_account_login(&self) -> Option<impl System<In = In<u32>, Out = ()> + 'static> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::OnLastIpUpdate in AC
    /// Called when an ip logged in successfully
    fn on_last_ip_update(&self) -> Option<impl System<In = In<(u32, String)>, Out = ()>> {
        input_script_non!(In<(u32, String)>)
    }
    /// AccountScript::OnFailedAccountLogin in TC/AC
    /// Called when an account login failed
    fn on_failed_account_login(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::OnEmailChange in TC/AC
    /// Called when Email is successfully changed for Account
    fn on_email_change(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::OnFailedEmailChange in TC/AC
    /// Called when Email failed to change for Account
    fn on_failed_email_change(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::OnPasswordChange in TC/AC
    /// Called when Password is successfully changed for Account
    fn on_password_change(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::OnFailedPasswordChange in TC/AC
    /// Called when Password failed to change for Account
    fn on_failed_password_change(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }
    /// AccountScript::CanAccountCreateCharacter in AC
    /// Called when creating a character on the Account
    fn can_account_create_character(&self) -> Option<impl System<In = In<(u32, u8, u8)>, Out = bool>> {
        input_script_non!(In<(u32, u8, u8)> => bool)
    }
}

#[derive(Component, Clone)]
pub struct AccountScriptObject {
    pub on_account_login:             Option<SystemId<In<u32>>>,
    pub on_last_ip_update:            Option<SystemId<In<(u32, String)>>>,
    pub on_failed_account_login:      Option<SystemId<In<u32>>>,
    pub on_email_change:              Option<SystemId<In<u32>>>,
    pub on_failed_email_change:       Option<SystemId<In<u32>>>,
    pub on_password_change:           Option<SystemId<In<u32>>>,
    pub on_failed_password_change:    Option<SystemId<In<u32>>>,
    pub can_account_create_character: Option<SystemId<In<(u32, u8, u8)>, bool>>,
}

impl<S: AccountScript> IntoScriptObject<S, AccountScriptObject> for S {
    fn create_from_systems(bevy_world: &mut World, s: &S) -> AccountScriptObject {
        AccountScriptObject {
            on_account_login:             s.on_account_login().map(|sys| bevy_world.register_system(sys)),
            on_last_ip_update:            s.on_last_ip_update().map(|sys| bevy_world.register_system(sys)),
            on_failed_account_login:      s.on_failed_account_login().map(|sys| bevy_world.register_system(sys)),
            on_email_change:              s.on_email_change().map(|sys| bevy_world.register_system(sys)),
            on_failed_email_change:       s.on_failed_email_change().map(|sys| bevy_world.register_system(sys)),
            on_password_change:           s.on_password_change().map(|sys| bevy_world.register_system(sys)),
            on_failed_password_change:    s.on_failed_password_change().map(|sys| bevy_world.register_system(sys)),
            can_account_create_character: s.can_account_create_character().map(|sys| bevy_world.register_system(sys)),
        }
    }
}

impl ScriptObjectTrait for AccountScriptObject {
    fn remove_systems_from_bevy(&self, bevy_world: &mut World) {
        _ = self.on_account_login.map(|s| bevy_world.unregister_system(s));
        _ = self.on_last_ip_update.map(|s| bevy_world.unregister_system(s));
        _ = self.on_failed_account_login.map(|s| bevy_world.unregister_system(s));
        _ = self.on_email_change.map(|s| bevy_world.unregister_system(s));
        _ = self.on_failed_email_change.map(|s| bevy_world.unregister_system(s));
        _ = self.on_password_change.map(|s| bevy_world.unregister_system(s));
        _ = self.on_failed_password_change.map(|s| bevy_world.unregister_system(s));
        _ = self.can_account_create_character.map(|s| bevy_world.unregister_system(s));
    }
}
