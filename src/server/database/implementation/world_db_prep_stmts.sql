-- :name sel_quest_pools
SELECT entry, pool_entry FROM pool_quest;

-- :name del_crelinked_respawn
DELETE FROM linked_respawn WHERE guid = ?;

-- :name rep_creature_linked_respawn
REPLACE INTO linked_respawn (guid, linkedGuid) VALUES (?, ?);

-- :name sel_creature_text
SELECT CreatureID, GroupID, ID, Text, Type, Language, Probability, Emote, Duration, Sound, BroadcastTextId, TextRange FROM creature_text;

-- :name sel_smart_scripts
SELECT entryorguid, source_type, id, link, event_type, event_phase_mask, event_chance, event_flags, event_param1, event_param2, event_param3, event_param4, event_param5, action_type, action_param1, action_param2, action_param3, action_param4, action_param5, action_param6, target_type, target_param1, target_param2, target_param3, target_param4, target_x, target_y, target_z, target_o FROM smart_scripts ORDER BY entryorguid, source_type, id, link;

-- :name sel_smartai_wp
SELECT entry, pointid, position_x, position_y, position_z, orientation, delay FROM waypoints ORDER BY entry, pointid;

-- :name del_gameobject
DELETE FROM gameobject WHERE guid = ?;

-- :name del_event_gameobject
DELETE FROM game_event_gameobject WHERE guid = ?;

-- :name ins_graveyard_zone
INSERT INTO graveyard_zone (ID, GhostZone, Faction) VALUES (?, ?, ?);

-- :name del_graveyard_zone
DELETE FROM graveyard_zone WHERE ID = ? AND GhostZone = ? AND Faction = ?;

-- :name ins_game_tele
INSERT INTO game_tele (id, position_x, position_y, position_z, orientation, map, name) VALUES (?, ?, ?, ?, ?, ?, ?);

-- :name del_game_tele
DELETE FROM game_tele WHERE name = ?;

-- :name ins_npc_vendor
INSERT INTO npc_vendor (entry, item, maxcount, incrtime, extendedcost) VALUES(?, ?, ?, ?, ?);

-- :name del_npc_vendor
DELETE FROM npc_vendor WHERE entry = ? AND item = ?;

-- :name sel_npc_vendor_ref
SELECT item, maxcount, incrtime, ExtendedCost FROM npc_vendor WHERE entry = ? ORDER BY slot ASC;

-- :name upd_creature_movement_type
UPDATE creature SET MovementType = ? WHERE guid = ?;

-- :name upd_creature_faction
UPDATE creature_template SET faction = ? WHERE entry = ?;

-- :name upd_creature_npcflag
UPDATE creature_template SET npcflag = ? WHERE entry = ?;

-- :name upd_creature_position
UPDATE creature SET position_x = ?, position_y = ?, position_z = ?, orientation = ? WHERE guid = ?;

-- :name upd_creature_wander_distance
UPDATE creature SET wander_distance = ?, MovementType = ? WHERE guid = ?;

-- :name upd_creature_spawn_time_secs
UPDATE creature SET spawntimesecs = ? WHERE guid = ?;

-- :name ins_creature_formation
INSERT INTO creature_formations (leaderGUID, memberGUID, dist, angle, groupAI) VALUES (?, ?, ?, ?, ?);

-- :name ins_waypoint_data
INSERT INTO waypoint_data (id, point, position_x, position_y, position_z) VALUES (?, ?, ?, ?, ?);

-- :name del_waypoint_data
DELETE FROM waypoint_data WHERE id = ? AND point = ?;

-- :name upd_waypoint_data_point
UPDATE waypoint_data SET point = point - 1 WHERE id = ? AND point > ?;

-- :name upd_waypoint_data_position
UPDATE waypoint_data SET position_x = ?, position_y = ?, position_z = ? where id = ? AND point = ?;

-- :name upd_waypoint_data_wpguid
UPDATE waypoint_data SET wpguid = ? WHERE id = ? and point = ?;

-- :name sel_waypoint_data_max_id
SELECT MAX(id) FROM waypoint_data;

-- :name sel_waypoint_data_max_point
SELECT MAX(point) FROM waypoint_data WHERE id = ?;

-- :name sel_waypoint_data_by_id
SELECT point, position_x, position_y, position_z, orientation, move_type, delay, action, action_chance FROM waypoint_data WHERE id = ? ORDER BY point;

-- :name sel_waypoint_data_pos_by_id
SELECT point, position_x, position_y, position_z FROM waypoint_data WHERE id = ?;

-- :name sel_waypoint_data_pos_first_by_id
SELECT position_x, position_y, position_z FROM waypoint_data WHERE point = 1 AND id = ?;

-- :name sel_waypoint_data_pos_last_by_id
SELECT position_x, position_y, position_z, orientation FROM waypoint_data WHERE id = ? ORDER BY point DESC LIMIT 1;

-- :name sel_waypoint_data_by_wpguid
SELECT id, point FROM waypoint_data WHERE wpguid = ?;

-- :name sel_waypoint_data_all_by_wpguid
SELECT id, point, delay, move_type, action, action_chance FROM waypoint_data WHERE wpguid = ?;

-- :name upd_waypoint_data_all_wpguid
UPDATE waypoint_data SET wpguid = 0;

-- :name sel_waypoint_data_by_pos
SELECT id, point FROM waypoint_data WHERE (abs(position_x - ?) <= ?) and (abs(position_y - ?) <= ?) and (abs(position_z - ?) <= ?);

-- :name sel_waypoint_data_wpguid_by_id
SELECT wpguid FROM waypoint_data WHERE id = ? and wpguid <> 0;

-- :name sel_waypoint_data_action
SELECT DISTINCT action FROM waypoint_data;

-- :name sel_waypoint_scripts_max_id
SELECT MAX(guid) FROM waypoint_scripts;

-- :name ins_creature_addon
INSERT INTO creature_addon(guid, path_id) VALUES (?, ?);

-- :name upd_creature_addon_path
UPDATE creature_addon SET path_id = ? WHERE guid = ?;

-- :name del_creature_addon
DELETE FROM creature_addon WHERE guid = ?;

-- :name sel_creature_addon_by_guid
SELECT guid FROM creature_addon WHERE guid = ?;

-- :name ins_waypoint_script
INSERT INTO waypoint_scripts (guid) VALUES (?);

-- :name del_waypoint_script
DELETE FROM waypoint_scripts WHERE guid = ?;

-- :name upd_waypoint_script_id
UPDATE waypoint_scripts SET id = ? WHERE guid = ?;

-- :name upd_waypoint_script_x
UPDATE waypoint_scripts SET x = ? WHERE guid = ?;

-- :name upd_waypoint_script_y
UPDATE waypoint_scripts SET y = ? WHERE guid = ?;

-- :name upd_waypoint_script_z
UPDATE waypoint_scripts SET z = ? WHERE guid = ?;

-- :name upd_waypoint_script_o
UPDATE waypoint_scripts SET o = ? WHERE guid = ?;

-- :name sel_waypoint_script_id_by_guid
SELECT id FROM waypoint_scripts WHERE guid = ?;

-- :name del_creature
DELETE FROM creature WHERE guid = ?;

-- :name sel_commands
SELECT name, security, help FROM command;

-- :name sel_creature_template
SELECT entry, difficulty_entry_1, difficulty_entry_2, difficulty_entry_3, KillCredit1, KillCredit2, modelid1, modelid2, modelid3, modelid4, name, subname, IconName, gossip_menu_id, minlevel, maxlevel, exp, faction, npcflag, speed_walk, speed_run, speed_swim, speed_flight, detection_range, scale, `rank`, dmgschool, DamageModifier, BaseAttackTime, RangeAttackTime, BaseVariance, RangeVariance, unit_class, unit_flags, unit_flags2, dynamicflags, family, trainer_type, trainer_spell, trainer_class, trainer_race, type, type_flags, lootid, pickpocketloot, skinloot, PetSpellDataId, VehicleId, mingold, maxgold, AIName, MovementType, ctm.Ground, ctm.Swim, ctm.Flight, ctm.Rooted, ctm.Chase, ctm.Random, ctm.InteractionPauseTimer, HoverHeight, HealthModifier, ManaModifier, ArmorModifier, ExperienceModifier, RacialLeader, movementId, RegenHealth, mechanic_immune_mask, spell_school_immune_mask, flags_extra, ScriptName FROM creature_template ct LEFT JOIN creature_template_movement ctm ON ct.entry = ctm.CreatureId WHERE entry = ?;

-- :name sel_waypoint_script_by_id
SELECT guid, delay, command, datalong, datalong2, dataint, x, y, z, o FROM waypoint_scripts WHERE id = ?;

-- :name sel_item_template_by_name
SELECT entry FROM item_template WHERE name = ?;

-- :name sel_creature_by_id
SELECT guid FROM creature WHERE id1 = ? OR id2 = ? OR id3 = ?;

-- :name sel_gameobject_nearest
SELECT guid, id, position_x, position_y, position_z, map, (POW(position_x - ?, 2) + POW(position_y - ?, 2) + POW(position_z - ?, 2)) AS order_ FROM gameobject WHERE map = ? AND (POW(position_x - ?, 2) + POW(position_y - ?, 2) + POW(position_z - ?, 2)) <= ? AND (phaseMask & ?) <> 0 ORDER BY order_;

-- :name sel_creature_nearest
SELECT guid, id1, id2, id3, position_x, position_y, position_z, map, (POW(position_x - ?, 2) + POW(position_y - ?, 2) + POW(position_z - ?, 2)) AS order_ FROM creature WHERE map = ? AND (POW(position_x - ?, 2) + POW(position_y - ?, 2) + POW(position_z - ?, 2)) <= ? AND (phaseMask & ?) <> 0 ORDER BY order_;

-- :name ins_creature
INSERT INTO creature (guid, id1, id2, id3, map, spawnMask, phaseMask, equipment_id, position_x, position_y, position_z, orientation, spawntimesecs, wander_distance, currentwaypoint, curhealth, curmana, MovementType, npcflag, unit_flags, dynamicflags) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_game_event_creature
DELETE FROM game_event_creature WHERE guid = ?;

-- :name del_game_event_model_equip
DELETE FROM game_event_model_equip WHERE guid = ?;

-- :name ins_gameobject
INSERT INTO gameobject (guid, id, map, spawnMask, phaseMask, position_x, position_y, position_z, orientation, rotation0, rotation1, rotation2, rotation3, spawntimesecs, animprogress, state) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name ins_disables
INSERT INTO disables (entry, sourceType, flags, comment) VALUES (?, ?, ?, ?);

-- :name sel_disables
SELECT entry FROM disables WHERE entry = ? AND sourceType = ?;

-- :name del_disables
DELETE FROM disables WHERE entry = ? AND sourceType = ?;

-- :name upd_creature_zone_area_data
UPDATE creature SET zoneId = ?, areaId = ? WHERE guid = ?;

-- :name upd_gameobject_zone_area_data
UPDATE gameobject SET zoneId = ?, areaId = ? WHERE guid = ?;

-- :name ins_gameobject_addon
INSERT INTO gameobject_addon (guid, invisibilityType, invisibilityValue) VALUES (?, 0, 0);

-- :name sel_req_xp
-- :doc 0: uint8
SELECT Experience FROM player_xp_for_level WHERE Level = ?;