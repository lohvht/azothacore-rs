use std::{collections::BTreeMap, sync::Mutex, time::Instant};

use azothacore_common::{
    az_error,
    bevy_app::{AzStartupFailedEvent, ToFromEntity, TokioRuntime},
    AzContext,
    AzResult,
};
use azothacore_database::{
    args_unwrap,
    database_env::{CharacterDatabase, CharacterPreparedStmts},
    DbDriver,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::{Bundle, Commands, Entity, EventWriter, Res, Resource},
};
use flagset::FlagSet;
use sqlx::{query, query_as, Transaction};
use tracing::{error, info, warn};

use crate::{
    game::{
        cache::character_cache::CharacterCache,
        dungeon_finding::{
            lfg_group_data::{KicksLeftConfig, LfgGroupData},
            LFGMgr,
        },
        entities::object::object_guid::{HighGuidParty, HighGuidPlayer, ObjectGuid, ObjectGuidRealmSpecific},
        groups::{Group, GroupDifficultySettings, GroupFlag, GroupFlags, GroupLootInfo, GroupMemberSlots, GroupRoundRobinLooterGuid, GroupTargetIcons},
        world::CurrentRealm,
    },
    shared::{
        data_stores::{db2_structure::Difficulty, DB2Storage},
        realms::Realm,
    },
};

#[derive(Resource, Default)]
struct GroupIDGenerator {
    /// Combination of GroupMgr::GroupDbStore and GroupMgr::NextGroupDbStoreId in TC
    group_db_ids: Mutex<BTreeMap<u32, Entity>>,
}

impl GroupIDGenerator {
    /// GroupMgr::RegisterGroupId in TC, GroupMgr::RegisterGroupDbStoreId in AC
    fn register_db_id(&self, db_id: u32, guid: ObjectGuid<HighGuidParty>) {
        let mut g = self.group_db_ids.lock().unwrap();
        g.insert(db_id, guid.to_entity());
    }

    /// GroupMgr::GenerateNewGroupDbStoreId in TC, GroupMgr::GenerateGroupId in AC
    fn next_db_id(&self, guid: ObjectGuid<HighGuidParty>) -> u32 {
        let mut g = self.group_db_ids.lock().unwrap();

        let group_id = g.last_entry().map(|v| *v.key()).unwrap_or(1);
        g.insert(group_id, guid.to_entity());
        group_id
    }

    fn group_guid(&self, db_id: u32) -> Option<ObjectGuid<HighGuidParty>> {
        let g = self.group_db_ids.lock().unwrap();
        g.get(&db_id).map(|v| ObjectGuid::from_entity(*v))
    }
}

/// NOTE: Mainly an adapted version of azerothcore's GroupMgr
#[derive(SystemParam)]
pub struct GroupMgr<'w> {
    id_gen: Res<'w, GroupIDGenerator>,
}

#[derive(sqlx::FromRow)]
struct DbGroupRes {
    #[sqlx(rename = "leaderGuid")]
    leader_guid:            u64,
    #[sqlx(rename = "lootMethod")]
    loot_method:            u8,
    #[sqlx(rename = "looterGuid")]
    looter_guid:            u64,
    #[sqlx(rename = "lootThreshold")]
    loot_threshold:         u8,
    icon1:                  Vec<u8>,
    icon2:                  Vec<u8>,
    icon3:                  Vec<u8>,
    icon4:                  Vec<u8>,
    icon5:                  Vec<u8>,
    icon6:                  Vec<u8>,
    icon7:                  Vec<u8>,
    icon8:                  Vec<u8>,
    #[sqlx(rename = "groupType")]
    group_type:             u16,
    difficulty:             u8,
    raiddifficulty:         u8,
    #[sqlx(rename = "legacyRaidDifficulty")]
    legacy_raid_difficulty: u8,
    #[sqlx(rename = "masterLooterGuid")]
    master_looter_guid:     u64,
    stored_id:              u32,
    dungeon:                u32,
    state:                  u8,
}

#[derive(Bundle)]
struct GroupLoadedBundle {
    group:        Group,
    flags:        GroupFlags,
    loot_info:    GroupLootInfo,
    member_slots: GroupMemberSlots,
    looter_guid:  GroupRoundRobinLooterGuid,
    icons:        GroupTargetIcons,
    difficulty:   GroupDifficultySettings,
}

impl DbGroupRes {
    /// Group::LoadGroupFromDB in TC / AC
    ///
    /// spawns the group from the DB record
    fn load<C: KicksLeftConfig>(
        self,
        current_realm: &Realm,
        character_cache: &CharacterCache,
        difficulty_store: &DB2Storage<Difficulty>,
        lfg_mgr: &LFGMgr<C>,
    ) -> AzResult<(GroupLoadedBundle, Option<LfgGroupData>)> {
        let Self {
            leader_guid,
            loot_method,
            looter_guid,
            loot_threshold,
            icon1,
            icon2,
            icon3,
            icon4,
            icon5,
            icon6,
            icon7,
            icon8,
            group_type,
            difficulty,
            raiddifficulty,
            legacy_raid_difficulty,
            master_looter_guid,
            stored_id,
            dungeon,
            state,
        } = self;
        let leader_guid = ObjectGuid::<HighGuidPlayer>::realm_specific(current_realm, leader_guid);

        // group leader not exist
        if character_cache.get_character_name_by_guid(leader_guid).is_none() {
            // // TODO: AC code, see if wanna adapt the cleanup on load. For now follow TC and just teturn
            // CharacterDatabaseTransaction trans = CharacterDatabase.BeginTransaction();
            // CharacterDatabasePreparedStatement* stmt = CharacterDatabase.GetPreparedStatement(CHAR_DEL_GROUP);
            // stmt->SetData(0, groupLowGuid);
            // trans->Append(stmt);
            // stmt = CharacterDatabase.GetPreparedStatement(CHAR_DEL_GROUP_MEMBER_ALL);
            // stmt->SetData(0, groupLowGuid);
            // trans->Append(stmt);
            // CharacterDatabase.CommitTransaction(trans);
            // stmt = CharacterDatabase.GetPreparedStatement(CHAR_DEL_LFG_DATA);
            // stmt->SetData(0, groupLowGuid);
            // CharacterDatabase.Execute(stmt);
            return Err(az_error!(
                "leader cannot be found in the character cache, Group DB ID={stored_id}, LeaderGUID={leader_guid}"
            ));
        }
        let Ok(loot_method) = loot_method.try_into() else {
            return Err(az_error!("wrong loot method, Group DB ID={stored_id}, method={loot_method}"));
        };
        let Ok(loot_threshold) = loot_threshold.try_into() else {
            return Err(az_error!("wrong loot threshold, Group DB ID={stored_id}, threshold={loot_threshold}"));
        };

        let group_flags = GroupFlags(FlagSet::new_truncated(group_type));
        let member_slots = {
            if group_flags.contains(GroupFlag::Raid) {
                GroupMemberSlots::new_raid()
            } else {
                GroupMemberSlots::new_party()
            }
        };

        let master_looter_guid = (master_looter_guid != 0).then(|| ObjectGuid::<HighGuidPlayer>::realm_specific(current_realm, master_looter_guid));
        let looter_guid = (looter_guid != 0).then(|| ObjectGuid::<HighGuidPlayer>::realm_specific(current_realm, looter_guid));
        let lfg_data = group_flags
            .contains(GroupFlag::Lfg)
            .then(|| {
                lfg_mgr
                    .new_lfg_group_data_from_dungeon_and_state(dungeon, state)
                    .map_err(|e| e.context(format!("loading LFG info failed: Group DB ID={stored_id}")))
            })
            .map_or(Ok(None), |v| v.map(Some))?;

        Ok((
            GroupLoadedBundle {
                group: Group {
                    leader_guid,
                    db_store_id: Some(stored_id),
                },
                flags: group_flags,
                loot_info: GroupLootInfo {
                    loot_method,
                    loot_threshold,
                    master_looter_guid,
                },
                looter_guid: GroupRoundRobinLooterGuid(looter_guid),
                icons: {
                    let mut ico = GroupTargetIcons::default();
                    ico[0] = icon1.as_slice().try_into().unwrap_or_default();
                    ico[1] = icon2.as_slice().try_into().unwrap_or_default();
                    ico[2] = icon3.as_slice().try_into().unwrap_or_default();
                    ico[3] = icon4.as_slice().try_into().unwrap_or_default();
                    ico[4] = icon5.as_slice().try_into().unwrap_or_default();
                    ico[5] = icon6.as_slice().try_into().unwrap_or_default();
                    ico[6] = icon7.as_slice().try_into().unwrap_or_default();
                    ico[7] = icon8.as_slice().try_into().unwrap_or_default();
                    ico
                },
                member_slots,
                difficulty: GroupDifficultySettings {
                    dungeon:     difficulty_store.check_loaded_dungeon_difficulty_id(difficulty.into()),
                    raid:        difficulty_store.check_loaded_raid_difficulty_id(raiddifficulty.into()),
                    legacy_raid: difficulty_store.check_loaded_legacy_raid_difficulty_id(legacy_raid_difficulty.into()),
                },
            },
            lfg_data,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct DbGroupMemberRes {
    stored_id:    u32,
    #[sqlx(rename = "memberGuid")]
    member_guid:  u64,
    #[sqlx(rename = "memberFlags")]
    member_flags: u8,
    subgroup:     u8,
    roles:        u8,
}

impl DbGroupMemberRes {
    /// Group::LoadMemberFromDB in TC / AC
    ///
    /// spawns the group member
    fn load<C: KicksLeftConfig>(
        self,
        group: &mut GroupLoadedBundle,
        lfg_info: Option<&mut LfgGroupData>,
        current_realm: &Realm,
        character_cache: &CharacterCache,
        lfg_mgr: &LFGMgr<C>,
    ) -> AzResult<()> {
        let Self {
            stored_id,
            member_guid,
            member_flags,
            subgroup,
            roles,
        } = self;
        let member_guid = ObjectGuid::<HighGuidPlayer>::realm_specific(current_realm, member_guid);
        if character_cache.get_character_name_by_guid(member_guid).is_none() {
            return Err(az_error!(
                "group member cannot be found in the character cache, group db_stored_id={stored_id}, memberGUID={member_guid}"
            ));
        }
        group
            .member_slots
            .add_member(member_guid, member_flags, subgroup.into(), roles)
            .with_context(|| "group member from DB cannot be added")?;

        // if let Some(lfg) = &mut group.lfg_data {
        //     // TODO: Complete LFG group setup if needed. Reference: https://github.com/TrinityCore/TrinityCore/blob/7.3.5/26972/src/server/game/Groups/Group.cpp#L237
        //     lfg_mgr.setup_group_member(member.guid, GetGUID());
        // }
        todo!()
        // LfgGroupData
    }
}

impl GroupMgr<'_> {
    /// GroupMgr::LoadGroups in TC / AC
    ///
    pub fn load<C: KicksLeftConfig>(
        mut commands: Commands,
        rt: Res<TokioRuntime>,
        char_db: Res<CharacterDatabase>,
        current_realm: Res<CurrentRealm>,
        character_cache: Res<CharacterCache>,
        difficulty_store: Res<DB2Storage<Difficulty>>,
        lfg_mgr: LFGMgr<C>,
        mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
    ) {
        let res: AzResult<()> = rt.block_on(async {
            let mut tx = char_db.begin().await.context("unable to begin transaction to init group manager")?;
            let mut group_bundles = Self::load_groups(&mut tx, &current_realm, &character_cache, &difficulty_store, &lfg_mgr).await?;
            Self::load_group_members(&mut tx, &mut group_bundles, &current_realm, &character_cache, &lfg_mgr).await?;
            // // TODO: TC CODE, IMPLEMENT GROUP INSTANCE SAVES!
            // TC_LOG_INFO("server.loading", "Loading Group instance saves...");
            // {
            //     uint32 oldMSTime = getMSTime();
            //     //                                                   0           1        2              3             4             5           6              7
            //     QueryResult result = CharacterDatabase.Query("SELECT gi.guid, i.map, gi.instance, gi.permanent, i.difficulty, i.resettime, i.entranceId, COUNT(g.guid) "
            //         "FROM group_instance gi INNER JOIN instance i ON gi.instance = i.id "
            //         "LEFT JOIN character_instance ci LEFT JOIN groups g ON g.leaderGuid = ci.guid ON ci.instance = gi.instance AND ci.permanent = 1 GROUP BY gi.instance ORDER BY gi.guid");
            //     if (!result)
            //     {
            //         TC_LOG_INFO("server.loading", ">> Loaded 0 group-instance saves. DB table `group_instance` is empty!");
            //         return;
            //     }

            //     uint32 count = 0;
            //     do
            //     {
            //         Field* fields = result->Fetch();
            //         Group* group = GetGroupByDbStoreId(fields[0].GetUInt32());
            //         // group will never be NULL (we have run consistency sql's before loading)

            //         MapEntry const* mapEntry = sMapStore.LookupEntry(fields[1].GetUInt16());
            //         if (!mapEntry || !mapEntry->IsDungeon())
            //         {
            //             TC_LOG_ERROR("sql.sql", "Incorrect entry in group_instance table : no dungeon map %d", fields[1].GetUInt16());
            //             continue;
            //         }

            //         uint32 diff = fields[4].GetUInt8();
            //         DifficultyEntry const* difficultyEntry = sDifficultyStore.LookupEntry(diff);
            //         if (!difficultyEntry || difficultyEntry->InstanceType != mapEntry->InstanceType)
            //             continue;

            //         InstanceSave* save = sInstanceSaveMgr->AddInstanceSave(mapEntry->ID, fields[2].GetUInt32(), Difficulty(diff), time_t(fields[5].GetUInt32()), fields[6].GetUInt32(), fields[7].GetUInt64() != 0, true);
            //         group->BindToInstance(save, fields[3].GetBool(), true);
            //         ++count;
            //     }
            //     while (result->NextRow());

            //     TC_LOG_INFO("server.loading", ">> Loaded %u group-instance saves in %u ms", count, GetMSTimeDiffToNow(oldMSTime));
            // }
            tx.commit().await?;

            // TODO: register the groups to be loaded.
            let group_id_generator = GroupIDGenerator::default();

            for (db_id, (group, lfg_data)) in group_bundles {
                let mut ec = commands.spawn(group);
                if let Some(lfg) = lfg_data {
                    ec.insert(lfg);
                }
                let e = ec.id();
                group_id_generator.register_db_id(db_id, ObjectGuid::from_entity(e));
            }
            commands.insert_resource(group_id_generator);
            Ok(())
        });

        if let Err(e) = res {
            error!(cause=?e, "group loading error");
            ev_startup_failed.send_default();
            return;
        }
        info!("Loaded groups successfully");
    }

    async fn load_groups<C: KicksLeftConfig>(
        chardb_tx: &mut Transaction<'_, DbDriver>,
        current_realm: &Realm,
        character_cache: &CharacterCache,
        difficulty_store: &DB2Storage<Difficulty>,
        lfg_mgr: &LFGMgr<'_, '_, C>,
    ) -> AzResult<BTreeMap<u32, (GroupLoadedBundle, Option<LfgGroupData>)>> {
        let old_ms_time = Instant::now();
        info!(target = "server.loading", "Loading Group Definitons...");
        // Delete all groups whose leader does not exist
        query("DELETE FROM `groups` WHERE leaderGuid NOT IN (SELECT guid FROM characters)")
            .execute(&mut **chardb_tx)
            .await
            .context("unable to clear stale leaderless groups")?;
        // Delete all groups with less than 2 members
        query("DELETE FROM `groups` WHERE guid NOT IN (SELECT guid FROM group_member GROUP BY guid HAVING COUNT(guid) > 1)")
            .execute(&mut **chardb_tx)
            .await
            .context("unable to remove groups with less than 2 members")?;
        // Delete invalid lfg_data
        let lfg_group_flags = (GroupFlag::Lfg & GroupFlag::LfgRestricted).bits();
        query("DELETE lfg_data FROM lfg_data LEFT JOIN `groups` ON lfg_data.guid = groups.guid WHERE groups.guid IS NULL OR groups.groupType <> ?")
            .bind(lfg_group_flags)
            .execute(&mut **chardb_tx)
            .await
            .context("unable to delete invalid LFG data")?;
        // group should be left so binds are cleared when disbanded
        // CharacterDatabase.DirectExecute("DELETE `groups` FROM `groups` LEFT JOIN lfg_data ON groups.guid = lfg_data.guid WHERE groups.groupType=12 AND lfg_data.guid IS NULL");

        let res = query_as::<_, DbGroupRes>(
            "SELECT
            g.leaderGuid, g.lootMethod, g.looterGuid, g.lootThreshold,
            g.icon1, g.icon2, g.icon3, g.icon4, g.icon5, g.icon6, g.icon7, g.icon8,
            g.groupType, g.difficulty, g.raiddifficulty, g.legacyRaidDifficulty,
            g.masterLooterGuid,
            g.stored_id,
            lfg.dungeon, lfg.state
        FROM
            groups g
        LEFT JOIN
            lfg_data lfg ON lfg.guid = g.guid
        ORDER BY g.guid ASC
    ",
        )
        .fetch_all(&mut **chardb_tx)
        .await?;
        let mut dbid_to_group = BTreeMap::new();
        let mut count = 0;
        for fields in res {
            let db_id = fields.stored_id;
            let group = match fields.load(current_realm, character_cache, difficulty_store, lfg_mgr) {
                Err(e) => {
                    error!(cause=?e, group_db_id=db_id, "group load failed");
                    continue;
                },
                Ok(v) => v,
            };
            dbid_to_group.insert(db_id, group);
            count += 1;
        }
        let elapsed = old_ms_time - Instant::now();
        info!(target = "server.loading", ">> Loaded {count} group definitions in {elapsed:?}");
        Ok(dbid_to_group)
    }

    async fn load_group_members<C: KicksLeftConfig>(
        chardb_tx: &mut Transaction<'_, DbDriver>,
        group_bundles: &mut BTreeMap<u32, (GroupLoadedBundle, Option<LfgGroupData>)>,
        current_realm: &Realm,
        character_cache: &CharacterCache,
        lfg_mgr: &LFGMgr<'_, '_, C>,
    ) -> AzResult<()> {
        let old_ms_time = Instant::now();

        info!(target = "server.loading", "Loading Group Members...");
        // Delete all rows from group_member or group_instance with no group
        query("DELETE FROM `group_member` WHERE guid NOT IN (SELECT guid FROM groups)")
            .execute(&mut **chardb_tx)
            .await
            .context("unable to clear group members with no group")?;
        query("DELETE FROM `group_instance` WHERE guid NOT IN (SELECT guid FROM groups)")
            .execute(&mut **chardb_tx)
            .await
            .context("unable to clear group instance with no group")?;
        // Delete all members that does not exist
        query("DELETE FROM `group_member` WHERE memberGuid NOT IN (SELECT guid FROM characters)")
            .execute(&mut **chardb_tx)
            .await
            .context("unable to clear group members with invalid members")?;

        let res = query_as::<_, DbGroupMemberRes>("SELECT stored_id, memberGuid, memberFlags, subgroup, roles FROM group_member ORDER BY guid")
            .fetch_all(&mut **chardb_tx)
            .await?;

        let mut count = 0;
        for fields in res {
            let db_id = fields.stored_id;
            let Some((ref mut group, ref mut lfg)) = group_bundles.get_mut(&db_id) else {
                warn!(
                    target = "misc",
                    "GroupMgr::LoadGroups: Consistency failed, can't find group (storage id: {db_id})"
                );
                continue;
            };
            let member_guid_low = fields.member_guid;
            if let Err(e) = fields.load(group, lfg.as_mut(), current_realm, character_cache, lfg_mgr) {
                warn!(cause=?e, "error loading group member {member_guid_low} for group stored ID {db_id}");
                _ = CharacterDatabase::del_group_member(&mut **chardb_tx, args_unwrap!(member_guid_low, db_id))
                    .await
                    .inspect_err(|e| {
                        warn!(cause=?e, "deleting group member {member_guid_low} for group stored ID {db_id}");
                    });
                continue;
            }
            count += 1;
        }
        let elapsed = old_ms_time - Instant::now();
        info!(target = "server.loading", ">> Loaded {count} group members in {elapsed:?}");
        Ok(())
    }
}
