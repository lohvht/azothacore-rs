use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicU32, AtomicU64},
    time::Instant,
};

use azothacore_common::{
    bevy_app::{AzStartupFailedEvent, TokioRuntime},
    deref_boilerplate,
    AzContext,
    AzResult,
};
use azothacore_database::{
    args,
    database_env::{CharacterDatabase, WorldDatabase},
};
use bevy::prelude::{Commands, EventWriter, In, Res, Resource};
use sqlx::{query_as, query_with};
use tracing::{error, info, warn};

use crate::{
    game::{
        battlegrounds::ArenaTeamIDGenerator,
        entities::object::object_guid::{HighGuidItem, HighGuidPlayer, HighGuidTransport, ObjectGuidLowGenerator},
        guilds::GuildIDGenerator,
    },
    shared::id_generators::{DBIDGenerator, IDGenerator, IDGeneratorTrait},
};

/// max allowed by client name length
pub const MAX_PLAYER_NAME: u8 = 12;
/// max server internal player name length (> MAX_PLAYER_NAME for support declined names)
pub const MAX_INTERNAL_PLAYER_NAME: u8 = 15;
/// max allowed by client name length
pub const MAX_PET_NAME: u8 = 12;
/// max allowed by client name length
pub const MAX_CHARTER_NAME: u8 = 24;

pub struct AuctionIDGenMarker;

/// ObjectMgr::_auctionId in TC / AC
pub type AuctionIDGenerator = IDGenerator<AuctionIDGenMarker, AtomicU32, u32>;

pub struct EquipmentSetGUIDGenMarker;

/// ObjectMgr::_equipmentSetGuid in TC / AC
pub type EquipmentSetGUIDGenerator = IDGenerator<EquipmentSetGUIDGenMarker, AtomicU64, u64>;

pub struct MailIDGenMarker;

/// ObjectMgr::_mailId in TC / AC
pub type MailIDGenerator = IDGenerator<MailIDGenMarker, AtomicU32, u32>;

pub struct HiPetNumberGenMarker;

/// ObjectMgr::_hiPetNumber in TC / AC
/// set in ObjectMgr::LoadPetNumber
pub type HiPetNumberGenerator = IDGenerator<HiPetNumberGenMarker, AtomicU32, u32>;

pub struct VoidItemIDGenMarker;

/// ObjectMgr::_voidItemId in TC
pub type VoidItemIDGenerator = IDGenerator<VoidItemIDGenMarker, AtomicU64, u64>;

pub struct CreatureSpawnIDGenMarker;

/// ObjectMgr::_creatureSpawnId in TC / AC
pub type CreatureSpawnIDGenerator = IDGenerator<CreatureSpawnIDGenMarker, AtomicU64, u64>;

pub struct GameObjectSpawnIDGenMarker;

/// ObjectMgr::_gameObjectSpawnId in TC / AC
pub type GameObjectSpawnIDGenerator = IDGenerator<GameObjectSpawnIDGenMarker, AtomicU64, u64>;

impl DBIDGenerator<CharacterDatabase, u32> for AuctionIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(id), 0) AS UNSIGNED INT)+1 FROM auctionhouse";
}
impl DBIDGenerator<CharacterDatabase, u64> for EquipmentSetGUIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "
    SELECT
        CAST(COALESCE(MAX(maxguid), 0) AS UNSIGNED INT)+1
    FROM (
        (
            SELECT MAX(setguid) AS maxguid FROM character_equipmentsets
        )
        UNION (
            SELECT MAX(setguid) AS maxguid FROM character_transmog_outfits
        )
    ) allsets";
}

impl DBIDGenerator<CharacterDatabase, u32> for MailIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(id), 0) AS UNSIGNED INT)+1 FROM mail";
}
impl DBIDGenerator<CharacterDatabase, u64> for VoidItemIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(itemId), 0) AS UNSIGNED INT)+1 from character_void_storage";
}
impl DBIDGenerator<WorldDatabase, u64> for CreatureSpawnIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guid), 0) AS UNSIGNED INT)+1 FROM creature";
}
impl DBIDGenerator<WorldDatabase, u64> for GameObjectSpawnIDGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guid), 0) AS UNSIGNED INT)+1 FROM gameobject";
}
impl DBIDGenerator<CharacterDatabase, u32> for HiPetNumberGenerator {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(id), 0) AS UNSIGNED INT)+1 FROM character_pet";
}

pub fn handle_set_highest_guids_error(In(res): In<AzResult<()>>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
    if let Err(e) = res {
        error!(target: "server::loading", cause=?e, "error initialising highest guid generators");
        ev_startup_failed.send_default();
    }
}

/// ObjectMgr::SetHighestGuids in TC / AC
pub fn set_highest_guids(mut commands: Commands, char_db: Res<CharacterDatabase>, world_db: Res<WorldDatabase>, rt: Res<TokioRuntime>) -> AzResult<()> {
    info!(target: "server::loading", "initialising GUID generators");

    commands
        .insert_resource(ObjectGuidLowGenerator::<HighGuidPlayer>::new_db_generator(&*char_db, &rt).with_context(|| "error retrieving max character guid")?);

    let guid_gen = ObjectGuidLowGenerator::<HighGuidItem>::new_db_generator(&*char_db, &rt).with_context(|| "error retrieving max item guid")?;
    let next_item_guid = guid_gen.next_after_max_used();
    rt.block_on(async {
        // Cleanup other tables from nonexistent guids ( >= _hiItemGuid)
        // all one time queries
        for q in [
            "DELETE FROM character_inventory WHERE item >= ?",
            "DELETE FROM mail_items WHERE item_guid >= ?",
            "DELETE FROM auctionhouse WHERE itemguid >= ?",
            "DELETE FROM guild_bank_item WHERE item_guid >= ?",
        ] {
            query_with(q, args!(next_item_guid)?)
                .execute(&**char_db)
                .await
                .with_context(|| format!("error clearing item guids for statement: '{q}' for item_guids above {next_item_guid}"))?;
        }
        AzResult::Ok(())
    })?;
    commands.insert_resource(guid_gen);

    commands.insert_resource(ObjectGuidLowGenerator::<HighGuidTransport>::new_db_generator(&*world_db, &rt).context("error retrieving max item guid")?);

    commands.insert_resource(AuctionIDGenerator::new_db_generator(&*char_db, &rt).context("error init AuctionIDGenerator in SetHighestGuids")?);
    commands.insert_resource(MailIDGenerator::new_db_generator(&*char_db, &rt).context("error init MailIDGenerator in SetHighestGuids")?);
    commands.insert_resource(ArenaTeamIDGenerator::new_db_generator(&*char_db, &rt).context("error init ArenaTeamIDGenerator in SetHighestGuids")?);
    commands.insert_resource(EquipmentSetGUIDGenerator::new_db_generator(&*char_db, &rt).context("error init EquipmentSetGUIDGenerator in SetHighestGuids")?);
    commands.insert_resource(GuildIDGenerator::new_db_generator(&*char_db, &rt).context("error init GuildIDGenerator in SetHighestGuids")?);
    commands.insert_resource(VoidItemIDGenerator::new_db_generator(&*char_db, &rt).context("error init VoidItemIDGenerator in SetHighestGuids")?);
    commands.insert_resource(CreatureSpawnIDGenerator::new_db_generator(&*world_db, &rt).context("error init CreatureSpawnIDGenerator in SetHighestGuids")?);
    commands
        .insert_resource(GameObjectSpawnIDGenerator::new_db_generator(&*world_db, &rt).context("error init GameObjectSpawnIDGenerator in SetHighestGuids")?);
    commands.insert_resource(HiPetNumberGenerator::new_db_generator(&*char_db, &rt).context("error init HiPetNumberGenerator in SetHighestGuids")?);

    Ok(())
}

/// equivalent to ObjectMgr::_scriptNamesStore in AC / TC
#[derive(Resource, Default)]
pub struct DBScriptNameStore(BTreeSet<String>);

deref_boilerplate!(DBScriptNameStore, BTreeSet<String>, 0);

/// equivalent to ObjectMgr::LoadScriptNames() in TC / AC
pub fn load_script_names(
    mut commands: Commands,
    rt: Res<TokioRuntime>,
    world_db: Res<WorldDatabase>,
    mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
) {
    info!(target:"server.loading", "Loading Script Names...");
    let mut name_store = DBScriptNameStore::default();
    let old_ms_time = Instant::now();
    let stmt = query_as::<_, (String,)>(
        "SELECT DISTINCT(ScriptName) FROM battleground_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM conversation_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM creature WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM creature_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM criteria_data WHERE ScriptName <> '' AND type = 11 \
        UNION 
        SELECT DISTINCT(ScriptName) FROM gameobject WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM gameobject_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM item_script_names WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM areatrigger_scripts WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM areatrigger_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM spell_script_names WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM transports WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM game_weather WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM conditions WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM outdoorpvp_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM scene_template WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(ScriptName) FROM quest_template_addon WHERE ScriptName <> '' \
        UNION 
        SELECT DISTINCT(script) FROM instance_template WHERE script <> ''",
    );
    let res = match rt.block_on(stmt.fetch_all(&**world_db)) {
        Err(e) => {
            ev_startup_failed.send_default();
            error!(cause=?e, "error retrieving script names");
            return;
        },
        Ok(r) if r.is_empty() => {
            warn!(target:"server.loading", ">> Loaded empty set of Script Names!");
            commands.insert_resource(name_store);
            return;
        },
        Ok(r) => r,
    };
    name_store.extend(res.into_iter().map(|(s,)| s));
    let elapsed = Instant::now() - old_ms_time;
    info!(target:"server.loading", ">> Loaded {} ScriptNames in {} ms", name_store.len(), elapsed.as_millis());
    commands.insert_resource(name_store);
}
