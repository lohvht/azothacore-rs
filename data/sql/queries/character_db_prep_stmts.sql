
-- :name del_quest_pool_save
DELETE FROM pool_quest_save WHERE pool_id = ?;

-- :name ins_quest_pool_save
INSERT INTO pool_quest_save (pool_id, quest_id) VALUES (?, ?);

-- :name del_nonexistent_guild_bank_item
DELETE FROM guild_bank_item WHERE guildid = ? AND TabId = ? AND SlotId = ?;

-- :name del_expired_bans
UPDATE character_banned SET active = 0 WHERE unbandate <= UNIX_TIMESTAMP() AND unbandate <> bandate;

-- :name sel_data_by_name
SELECT guid, account, name, gender, race, class, level FROM characters WHERE deleteDate IS NULL AND name = ?;

-- :name sel_data_by_guid
SELECT guid, account, name, gender, race, class, level FROM characters WHERE deleteDate IS NULL AND guid = ?;

-- :name sel_check_name
SELECT 1 FROM characters WHERE name = ?;

-- :name sel_check_guid
SELECT 1 FROM characters WHERE guid = ?;

-- :name sel_sum_chars
SELECT COUNT(guid) FROM characters WHERE account = ?;

-- :name sel_char_create_info
SELECT level, race, class FROM characters WHERE account = ? LIMIT 0, ?;

-- :name ins_character_ban
INSERT INTO character_banned VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, ?, ?, 1);

-- :name upd_character_ban
UPDATE character_banned SET active = 0 WHERE guid = ? AND active != 0;

-- :name del_character_ban
DELETE cb FROM character_banned cb INNER JOIN characters c ON c.guid = cb.guid WHERE c.account = ?;

-- :name sel_baninfo
SELECT FROM_UNIXTIME(bandate, '%Y-%m-%d %H:%i:%s'), unbandate-bandate, active, unbandate, banreason, bannedby FROM character_banned WHERE guid = ? ORDER BY bandate ASC;

-- :name sel_guid_by_name_filter
SELECT guid, name FROM characters WHERE name LIKE CONCAT('%%', ?, '%%');

-- :name sel_baninfo_list
SELECT bandate, unbandate, bannedby, banreason FROM character_banned WHERE guid = ? ORDER BY unbandate;

-- :name sel_banned_name
SELECT characters.name FROM characters, character_banned WHERE character_banned.guid = ? AND character_banned.guid = characters.guid;

-- :name sel_enum
SELECT c.guid, c.name, c.race, c.class, c.gender, c.skin, c.face, c.hairStyle, c.hairColor, c.facialStyle, c.level, c.zone, c.map, c.position_x, c.position_y, c.position_z,
  gm.guildid, c.playerFlags, c.at_login, cp.entry, cp.modelid, cp.level, c.equipmentCache, cb.guid, c.extra_flags
  FROM characters AS c LEFT JOIN character_pet AS cp ON c.guid = cp.owner AND cp.slot = ? LEFT JOIN guild_member AS gm ON c.guid = gm.guid
  LEFT JOIN character_banned AS cb ON c.guid = cb.guid AND cb.active = 1 WHERE c.account = ? AND c.deleteInfos_Name IS NULL ORDER BY COALESCE(c.order, c.guid);


-- :name sel_enum_declined_name
SELECT c.guid, c.name, c.race, c.class, c.gender, c.skin, c.face, c.hairStyle, c.hairColor, c.facialStyle, c.level, c.zone, c.map, 
  c.position_x, c.position_y, c.position_z, gm.guildid, c.playerFlags, c.at_login, cp.entry, cp.modelid, cp.level, c.equipmentCache, 
  cb.guid, c.extra_flags, cd.genitive FROM characters AS c LEFT JOIN character_pet AS cp ON c.guid = cp.owner AND cp.slot = ? 
  LEFT JOIN character_declinedname AS cd ON c.guid = cd.guid LEFT JOIN guild_member AS gm ON c.guid = gm.guid 
  LEFT JOIN character_banned AS cb ON c.guid = cb.guid AND cb.active = 1 WHERE c.account = ? AND c.deleteInfos_Name IS NULL ORDER BY COALESCE(c.order, c.guid);

-- :name sel_free_name
SELECT guid, name, at_login FROM characters WHERE guid = ? AND account = ? AND NOT EXISTS (SELECT NULL FROM characters WHERE name = ?);

-- :name sel_char_zone
SELECT zone FROM characters WHERE guid = ?;

-- :name sel_character_name_data
SELECT race, class, gender, level FROM characters WHERE guid = ?;

-- :name sel_char_position_xyz
SELECT map, position_x, position_y, position_z FROM characters WHERE guid = ?;

-- :name sel_char_position
SELECT position_x, position_y, position_z, orientation, map, taxi_path FROM characters WHERE guid = ?;

-- :name del_quest_status_daily
DELETE FROM character_queststatus_daily;

-- :name del_quest_status_weekly
DELETE FROM character_queststatus_weekly;

-- :name del_quest_status_monthly
DELETE FROM character_queststatus_monthly;

-- :name del_quest_status_seasonal
DELETE FROM character_queststatus_seasonal WHERE event = ?;

-- :name del_quest_status_daily_char
DELETE FROM character_queststatus_daily WHERE guid = ?;

-- :name del_quest_status_weekly_char
DELETE FROM character_queststatus_weekly WHERE guid = ?;

-- :name del_quest_status_monthly_char
DELETE FROM character_queststatus_monthly WHERE guid = ?;

-- :name del_quest_status_seasonal_char
DELETE FROM character_queststatus_seasonal WHERE guid = ?;

-- :name del_battleground_random
DELETE FROM character_battleground_random;

-- :name ins_battleground_random
INSERT INTO character_battleground_random (guid) VALUES (?);

-- >>>>> Start LoginQueryHolder content

-- :name sel_character
SELECT guid, account, name, race, class, gender, level, xp, money, skin, face, hairStyle, hairColor, facialStyle, bankSlots, restState, playerFlags, 
  position_x, position_y, position_z, map, orientation, taximask, cinematic, totaltime, leveltime, rest_bonus, logout_time, is_logout_resting, resettalents_cost, 
  resettalents_time, trans_x, trans_y, trans_z, trans_o, transguid, extra_flags, stable_slots, at_login, zone, online, death_expire_time, taxi_path, instance_mode_mask, 
  arenaPoints, totalHonorPoints, todayHonorPoints, yesterdayHonorPoints, totalKills, todayKills, yesterdayKills, chosenTitle, knownCurrencies, watchedFaction, drunk, 
  health, power1, power2, power3, power4, power5, power6, power7, instance_id, talentGroupsCount, activeTalentGroup, exploredZones, equipmentCache, ammoId, 
  knownTitles, actionBars, grantableLevels, innTriggerId FROM characters WHERE guid = ?;


-- :name sel_character_auras
SELECT casterGuid, itemGuid, spell, effectMask, recalculateMask, stackCount, amount0, amount1, amount2, 
  base_amount0, base_amount1, base_amount2, maxDuration, remainTime, remainCharges FROM character_aura WHERE guid = ?;

-- :name sel_character_spell
SELECT spell, specMask FROM character_spell WHERE guid = ?;

-- :name sel_character_queststatus
SELECT quest, status, explored, timer, mobcount1, mobcount2, mobcount3, mobcount4, 
  itemcount1, itemcount2, itemcount3, itemcount4, itemcount5, itemcount6, playercount FROM character_queststatus WHERE guid = ? AND status <> 0;

-- :name sel_character_dailyqueststatus
SELECT quest, time FROM character_queststatus_daily WHERE guid = ?;

-- :name sel_character_weeklyqueststatus
SELECT quest FROM character_queststatus_weekly WHERE guid = ?;

-- :name sel_character_monthlyqueststatus
SELECT quest FROM character_queststatus_monthly WHERE guid = ?;

-- :name sel_character_seasonalqueststatus
SELECT quest, event FROM character_queststatus_seasonal WHERE guid = ?;

-- :name ins_character_dailyqueststatus
INSERT INTO character_queststatus_daily (guid, quest, time) VALUES (?, ?, ?);

-- :name ins_character_weeklyqueststatus
INSERT INTO character_queststatus_weekly (guid, quest) VALUES (?, ?);

-- :name ins_character_monthlyqueststatus
INSERT INTO character_queststatus_monthly (guid, quest) VALUES (?, ?);

-- :name ins_character_seasonalqueststatus
INSERT IGNORE INTO character_queststatus_seasonal (guid, quest, event) VALUES (?, ?, ?);

-- :name sel_character_reputation
SELECT faction, standing, flags FROM character_reputation WHERE guid = ?;

-- :name sel_character_inventory
SELECT creatorGuid, giftCreatorGuid, count, duration, charges, flags, enchantments, randomPropertyId, durability, playedTime, text, bag, slot, 
  item, itemEntry FROM character_inventory ci JOIN item_instance ii ON ci.item = ii.guid WHERE ci.guid = ? ORDER BY bag, slot;

-- :name sel_character_actions
SELECT a.button, a.action, a.type FROM character_action as a, characters as c WHERE a.guid = c.guid AND a.spec = c.activeTalentGroup AND a.guid = ? ORDER BY button;

-- :name sel_character_mailcount_unread
SELECT COUNT(id) FROM mail WHERE receiver = ? AND (checked & 1) = 0 AND deliver_time <= ?;

-- :name sel_character_mailcount_unread_synch
SELECT COUNT(id) FROM mail WHERE receiver = ? AND (checked & 1) = 0 AND deliver_time <= ?;

-- :name sel_mail_server_character
SELECT mailId from mail_server_character WHERE guid = ? and mailId = ?;

-- :name rep_mail_server_character
REPLACE INTO mail_server_character (guid, mailId) values (?, ?);

-- :name sel_character_sociallist
SELECT friend, flags, note FROM character_social JOIN characters ON characters.guid = character_social.friend WHERE character_social.guid = ? AND deleteinfos_name IS NULL LIMIT 255;

-- :name sel_character_homebind
SELECT mapId, zoneId, posX, posY, posZ, posO FROM character_homebind WHERE guid = ?;

-- :name sel_character_spellcooldowns
SELECT spell, category, item, time, needSend FROM character_spell_cooldown WHERE guid = ?;

-- :name sel_character_declinednames
SELECT genitive, dative, accusative, instrumental, prepositional FROM character_declinedname WHERE guid = ?;

-- :name sel_character_achievements
SELECT achievement, date FROM character_achievement WHERE guid = ?;

-- :name sel_character_criteriaprogress
SELECT criteria, counter, date FROM character_achievement_progress WHERE guid = ?;

-- :name sel_character_equipmentsets
SELECT setguid, setindex, name, iconname, ignore_mask, item0, item1, item2, item3, item4, item5, item6, item7, item8, 
  item9, item10, item11, item12, item13, item14, item15, item16, item17, item18 FROM character_equipmentsets WHERE guid = ? ORDER BY setindex;

-- :name sel_character_entry_point
SELECT joinX, joinY, joinZ, joinO, joinMapId, taxiPath0, taxiPath1, mountSpell FROM character_entry_point WHERE guid = ?;

-- :name sel_character_glyphs
SELECT talentGroup, glyph1, glyph2, glyph3, glyph4, glyph5, glyph6 FROM character_glyphs WHERE guid = ?;

-- :name sel_character_talents
SELECT spell, specMask FROM character_talent WHERE guid = ?;

-- :name sel_character_skills
SELECT skill, value, max FROM character_skills WHERE guid = ?;

-- :name sel_character_randombg
SELECT guid FROM character_battleground_random WHERE guid = ?;

-- :name sel_character_banned
SELECT guid FROM character_banned WHERE guid = ? AND active = 1;

-- :name sel_character_queststatusrew
SELECT quest FROM character_queststatus_rewarded WHERE guid = ? AND active = 1;

-- :name sel_account_instancelocktimes
SELECT instanceId, releaseTime FROM account_instance_times WHERE accountId = ?;

-- :name sel_brew_of_the_month
SELECT lastEventId FROM character_brew_of_the_month WHERE guid = ?;

-- :name rep_brew_of_the_month
REPLACE INTO character_brew_of_the_month (guid, lastEventId) VALUES (?, ?);

-- >>>>> End LoginQueryHolder content


-- :name sel_character_actions_spec
SELECT button, action, type FROM character_action WHERE guid = ? AND spec = ? ORDER BY button;

-- :name sel_mailitems
SELECT creatorGuid, giftCreatorGuid, count, duration, charges, flags, enchantments, randomPropertyId, durability, playedTime, text, item_guid, itemEntry, ii.owner_guid, m.id FROM mail_items mi INNER JOIN mail m ON mi.mail_id = m.id LEFT JOIN item_instance ii ON mi.item_guid = ii.guid WHERE m.receiver = ?;

-- :name sel_auction_items
SELECT creatorGuid, giftCreatorGuid, count, duration, charges, flags, enchantments, randomPropertyId, durability, playedTime, text, itemguid, itemEntry FROM auctionhouse ah JOIN item_instance ii ON ah.itemguid = ii.guid;

-- :name sel_auctions
SELECT id, houseid, itemguid, itemEntry, count, itemowner, buyoutprice, time, buyguid, lastbid, startbid, deposit FROM auctionhouse ah INNER JOIN item_instance ii ON ii.guid = ah.itemguid;

-- :name ins_auction
INSERT INTO auctionhouse (id, houseid, itemguid, itemowner, buyoutprice, time, buyguid, lastbid, startbid, deposit) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_auction
DELETE FROM auctionhouse WHERE id = ?;

-- :name upd_auction_bid
UPDATE auctionhouse SET buyguid = ?, lastbid = ? WHERE id = ?;

-- :name ins_mail
INSERT INTO mail(id, messageType, stationery, mailTemplateId, sender, receiver, subject, body, has_items, expire_time, deliver_time, money, cod, checked) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_mail_by_id
DELETE FROM mail WHERE id = ?;

-- :name ins_mail_item
INSERT INTO mail_items(mail_id, item_guid, receiver) VALUES (?, ?, ?);

-- :name del_mail_item
DELETE FROM mail_items WHERE item_guid = ?;

-- :name del_invalid_mail_item
DELETE FROM mail_items WHERE item_guid = ?;

-- :name sel_expired_mail
SELECT id, messageType, sender, receiver, has_items, expire_time, stationery, checked, mailTemplateId FROM mail WHERE expire_time < ?;

-- :name sel_expired_mail_items
SELECT item_guid, itemEntry, mail_id FROM mail_items mi INNER JOIN item_instance ii ON ii.guid = mi.item_guid LEFT JOIN mail mm ON mi.mail_id = mm.id WHERE mm.id IS NOT NULL AND mm.expire_time < ?;

-- :name upd_mail_returned
UPDATE mail SET sender = ?, receiver = ?, expire_time = ?, deliver_time = ?, cod = 0, checked = ? WHERE id = ?;

-- :name upd_mail_item_receiver
UPDATE mail_items SET receiver = ? WHERE item_guid = ?;

-- :name upd_item_owner
UPDATE item_instance SET owner_guid = ? WHERE guid = ?;

-- :name sel_item_refunds
SELECT player_guid, paidMoney, paidExtendedCost FROM item_refund_instance WHERE item_guid = ? AND player_guid = ? LIMIT 1;

-- :name sel_item_bop_trade
SELECT allowedPlayers FROM item_soulbound_trade_data WHERE itemGuid = ? LIMIT 1;

-- :name del_item_bop_trade
DELETE FROM item_soulbound_trade_data WHERE itemGuid = ? LIMIT 1;

-- :name ins_item_bop_trade
INSERT INTO item_soulbound_trade_data VALUES (?, ?);

-- :name rep_inventory_item
REPLACE INTO character_inventory (guid, bag, slot, item) VALUES (?, ?, ?, ?);

-- :name rep_item_instance
REPLACE INTO item_instance (itemEntry, owner_guid, creatorGuid, giftCreatorGuid, count, duration, charges, flags, enchantments, randomPropertyId, durability, playedTime, text, guid) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name upd_item_instance
UPDATE item_instance SET itemEntry = ?, owner_guid = ?, creatorGuid = ?, giftCreatorGuid = ?, count = ?, duration = ?, charges = ?, flags = ?, enchantments = ?, randomPropertyId = ?, durability = ?, playedTime = ?, text = ? WHERE guid = ?;

-- :name upd_item_instance_on_load
UPDATE item_instance SET duration = ?, flags = ?, durability = ? WHERE guid = ?;

-- :name del_item_instance
DELETE FROM item_instance WHERE guid = ?;

-- :name del_item_instance_by_owner
DELETE FROM item_instance WHERE owner_guid = ?;

-- :name upd_gift_owner
UPDATE character_gifts SET guid = ? WHERE item_guid = ?;

-- :name del_gift
DELETE FROM character_gifts WHERE item_guid = ?;

-- :name sel_character_gift_by_item
SELECT entry, flags FROM character_gifts WHERE item_guid = ?;

-- :name sel_account_by_name
SELECT account FROM characters WHERE name = ?;

-- :name del_account_instance_lock_times
DELETE FROM account_instance_times WHERE accountId = ?;

-- :name ins_account_instance_lock_times
INSERT INTO account_instance_times (accountId, instanceId, releaseTime) VALUES (?, ?, ?);

-- :name sel_match_maker_rating
SELECT matchMakerRating, maxMMR  FROM character_arena_stats WHERE guid = ? AND slot = ?;

-- :name sel_character_count
SELECT account, COUNT(guid) FROM characters WHERE account = ? GROUP BY account;

-- :name upd_name_by_guid
UPDATE characters SET name = ? WHERE guid = ?;

-- :name del_declined_name
DELETE FROM character_declinedname WHERE guid = ?;

-- >>>>> Guild handling

-- :name ins_guild
-- :doc 0: uint32, 1: string, 2: uint32, 3: string, 4: string, 5: uint64, 6-10: uint32, 11: uint64
INSERT INTO guild (guildid, name, leaderguid, info, motd, createdate, EmblemStyle, EmblemColor, BorderStyle, BorderColor, BackgroundColor, BankMoney) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_guild
-- :doc 0: uint32
DELETE FROM guild WHERE guildid = ?;

-- :name upd_guild_name
-- :doc 0: string, 1: uint32
UPDATE guild SET name = ? WHERE guildid = ?;


-- :name ins_guild_member
-- :doc 0: uint32, 1: uint32, 2: uint8, 4: string, 5: string
INSERT INTO guild_member (guildid, guid, `rank`, pnote, offnote) VALUES (?, ?, ?, ?, ?);

-- :name del_guild_member
-- :doc 0: uint32
DELETE FROM guild_member WHERE guid = ?;

-- :name del_guild_members
-- :doc 0: uint32
DELETE FROM guild_member WHERE guildid = ?;

-- :name sel_guild_member_extended
SELECT g.guildid, g.name, gr.rname, gm.pnote, gm.offnote 
                  FROM guild g JOIN guild_member gm ON g.guildid = gm.guildid 
                  JOIN guild_rank gr ON g.guildid = gr.guildid AND gm.`rank` = gr.rid WHERE gm.guid = ?;

-- :name ins_guild_rank
-- :doc 0: uint32, 1: uint8, 3: string, 4: uint32, 5: uint32
INSERT INTO guild_rank (guildid, rid, rname, rights, BankMoneyPerDay) VALUES (?, ?, ?, ?, ?);

-- :name del_guild_ranks
-- :doc 0: uint32
DELETE FROM guild_rank WHERE guildid = ?;

-- :name del_guild_lowest_rank
-- :doc 0: uint32, 1: uint8
DELETE FROM guild_rank WHERE guildid = ? AND rid >= ?;

-- :name ins_guild_bank_tab
-- :doc 0: uint32, 1: uint8
INSERT INTO guild_bank_tab (guildid, TabId) VALUES (?, ?);

-- :name del_guild_bank_tab
-- :doc 0: uint32, 1: uint8
DELETE FROM guild_bank_tab WHERE guildid = ? AND TabId = ?;

-- :name del_guild_bank_tabs
-- :doc 0: uint32
DELETE FROM guild_bank_tab WHERE guildid = ?;

-- :name ins_guild_bank_item
-- :doc 0: uint32, 1: uint8, 2: uint8, 3: uint32, 4: uint32
INSERT INTO guild_bank_item (guildid, TabId, SlotId, item_guid) VALUES (?, ?, ?, ?);

-- :name del_guild_bank_item
-- :doc 0: uint32, 1: uint8, 2: uint8
DELETE FROM guild_bank_item WHERE guildid = ? AND TabId = ? AND SlotId = ?;

-- :name del_guild_bank_items
-- :doc 0: uint32
DELETE FROM guild_bank_item WHERE guildid = ?;

-- :name ins_guild_bank_right
-- :doc 0: uint32, 1: uint8, 2: uint8, 3: uint8, 4: uint32
INSERT INTO guild_bank_right (guildid, TabId, rid, gbright, SlotPerDay) VALUES (?, ?, ?, ?, ?) 
                  ON DUPLICATE KEY UPDATE gbright = VALUES(gbright), SlotPerDay = VALUES(SlotPerDay);

-- :name del_guild_bank_rights
-- :doc 0: uint32
DELETE FROM guild_bank_right WHERE guildid = ?;

-- :name del_guild_bank_rights_for_rank
-- :doc 0: uint32, 1: uint8
DELETE FROM guild_bank_right WHERE guildid = ? AND rid = ?;

-- :name ins_guild_bank_eventlog
-- :doc 0-1: uint32, 2-3: uint8, 4-5: uint32, 6: uint16, 7: uint8, 8: uint64
INSERT INTO guild_bank_eventlog (guildid, LogGuid, TabId, EventType, PlayerGuid, ItemOrMoney, ItemStackCount, DestTabId, TimeStamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_guild_bank_eventlog
-- :doc 0: uint32, 1: uint32, 2: uint8
DELETE FROM guild_bank_eventlog WHERE guildid = ? AND LogGuid = ? AND TabId = ?;

-- :name del_guild_bank_eventlogs
-- :doc 0: uint32
DELETE FROM guild_bank_eventlog WHERE guildid = ?;

-- :name ins_guild_eventlog
-- :doc 0-1: uint32, 2: uint8, 3-4: uint32, 5: uint8, 6: uint64
INSERT INTO guild_eventlog (guildid, LogGuid, EventType, PlayerGuid1, PlayerGuid2, NewRank, TimeStamp) VALUES (?, ?, ?, ?, ?, ?, ?);

-- :name del_guild_eventlog
-- :doc 0: uint32, 1: uint32
DELETE FROM guild_eventlog WHERE guildid = ? AND LogGuid = ?;

-- :name del_guild_eventlogs
-- :doc 0: uint32
DELETE FROM guild_eventlog WHERE guildid = ?;

-- :name upd_guild_member_pnote
-- :doc 0: string, 1: uint32
UPDATE guild_member SET pnote = ? WHERE guid = ?;

-- :name upd_guild_member_offnote
-- :doc 0: string, 1: uint32
UPDATE guild_member SET offnote = ? WHERE guid = ?;

-- :name upd_guild_member_rank
-- :doc 0: uint8, 1: uint32
UPDATE guild_member SET `rank` = ? WHERE guid = ?;

-- :name upd_guild_motd
-- :doc 0: string, 1: uint32
UPDATE guild SET motd = ? WHERE guildid = ?;

-- :name upd_guild_info
-- :doc 0: string, 1: uint32
UPDATE guild SET info = ? WHERE guildid = ?;

-- :name upd_guild_leader
-- :doc 0: uint32, 1: uint32
UPDATE guild SET leaderguid = ? WHERE guildid = ?;

-- :name upd_guild_rank_name
-- :doc 0: string, 1: uint8, 2: uint32
UPDATE guild_rank SET rname = ? WHERE rid = ? AND guildid = ?;

-- :name upd_guild_rank_rights
-- :doc 0: uint32, 1: uint8, 2: uint32
UPDATE guild_rank SET rights = ? WHERE rid = ? AND guildid = ?;

-- :name upd_guild_emblem_info
-- :doc 0-5: uint32
UPDATE guild SET EmblemStyle = ?, EmblemColor = ?, BorderStyle = ?, BorderColor = ?, BackgroundColor = ? WHERE guildid = ?;

-- :name upd_guild_bank_tab_info
-- :doc 0: string, 1: string, 2: uint32, 3: uint8
UPDATE guild_bank_tab SET TabName = ?, TabIcon = ? WHERE guildid = ? AND TabId = ?;

-- :name upd_guild_bank_money
-- :doc 0: uint64, 1: uint32
UPDATE guild SET BankMoney = ? WHERE guildid = ?;

-- :name upd_guild_bank_eventlog_tab
-- :doc 0: uint8, 1: uint32, 2: uint8, 3: uint32
UPDATE guild_bank_eventlog SET TabId = ? WHERE guildid = ? AND TabId = ? AND LogGuid = ?;

-- :name upd_guild_rank_bank_money
-- :doc 0: uint32, 1: uint8, 2: uint32
UPDATE guild_rank SET BankMoneyPerDay = ? WHERE rid = ? AND guildid = ?;

-- :name upd_guild_bank_tab_text
-- :doc 0: string, 1: uint32, 2: uint8
UPDATE guild_bank_tab SET TabText = ? WHERE guildid = ? AND TabId = ?;


-- :name ins_guild_member_withdraw
INSERT INTO guild_member_withdraw (guid, tab0, tab1, tab2, tab3, tab4, tab5, money) VALUES (?, ?, ?, ?, ?, ?, ?, ?) 
  ON DUPLICATE KEY UPDATE tab0 = VALUES (tab0), tab1 = VALUES (tab1), tab2 = VALUES (tab2), tab3 = VALUES (tab3), tab4 = VALUES (tab4), tab5 = VALUES (tab5);

-- :name del_guild_member_withdraw
TRUNCATE guild_member_withdraw;


-- :name sel_char_data_for_guild
-- :doc 0: uint32, 1: uint32, 2: uint32
SELECT name, level, class, gender, zone, account FROM characters WHERE guid = ?;

-- >>>>> Chat channel handling

-- :name ins_channel
INSERT INTO channels(channelId, name, team, announce, lastUsed) VALUES (?, ?, ?, ?, UNIX_TIMESTAMP());

-- :name upd_channel
UPDATE channels SET announce = ?, password = ?, lastUsed = UNIX_TIMESTAMP() WHERE channelId = ?;

-- :name del_channel
DELETE FROM channels WHERE name = ? AND team = ?;

-- :name upd_channel_usage
UPDATE channels SET lastUsed = UNIX_TIMESTAMP() WHERE channelId = ?;

-- :name del_old_channels
DELETE FROM channels WHERE lastUsed + ? < UNIX_TIMESTAMP();

-- :name del_old_channels_bans
DELETE cb.* FROM channels_bans cb LEFT JOIN channels cn ON cb.channelId=cn.channelId WHERE cn.channelId IS NULL OR cb.banTime <= UNIX_TIMESTAMP();

-- :name ins_channel_ban
REPLACE INTO channels_bans VALUES (?, ?, ?);

-- :name del_channel_ban
DELETE FROM channels_bans WHERE channelId = ? AND playerGUID = ?;

-- >>>>> Equipmentsets

-- :name upd_equip_set
UPDATE character_equipmentsets SET name=?, iconname=?, ignore_mask=?, item0=?, item1=?, item2=?, item3=?, 
  item4=?, item5=?, item6=?, item7=?, item8=?, item9=?, item10=?, item11=?, item12=?, item13=?, item14=?, item15=?, item16=?, 
  item17=?, item18=? WHERE guid=? AND setguid=? AND setindex=?;

-- :name ins_equip_set
INSERT INTO character_equipmentsets (guid, setguid, setindex, name, iconname, ignore_mask, item0, item1, item2, item3, 
  item4, item5, item6, item7, item8, item9, item10, item11, item12, item13, item14, item15, item16, item17, item18) 
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_equip_set
DELETE FROM character_equipmentsets WHERE setguid=?;

-- >>>>> Auras

-- :name ins_aura
INSERT INTO character_aura (guid, casterGuid, itemGuid, spell, effectMask, recalculateMask, stackcount, amount0, amount1, amount2, base_amount0, base_amount1, base_amount2, maxDuration, remainTime, remainCharges) 
                  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- >>>>> Account data

-- :name sel_account_data
SELECT type, time, data FROM account_data WHERE accountId = ?;

-- :name rep_account_data
REPLACE INTO account_data (accountId, type, time, data) VALUES (?, ?, ?, ?);

-- :name del_account_data
DELETE FROM account_data WHERE accountId = ?;

-- :name sel_player_account_data
SELECT type, time, data FROM character_account_data WHERE guid = ?;

-- :name rep_player_account_data
REPLACE INTO character_account_data(guid, type, time, data) VALUES (?, ?, ?, ?);

-- :name del_player_account_data
DELETE FROM character_account_data WHERE guid = ?;

-- >>>>> Tutorials

-- :name sel_tutorials
SELECT tut0, tut1, tut2, tut3, tut4, tut5, tut6, tut7 FROM account_tutorial WHERE accountId = ?;

-- :name sel_has_tutorials
SELECT 1 FROM account_tutorial WHERE accountId = ?;

-- :name ins_tutorials
INSERT INTO account_tutorial(tut0, tut1, tut2, tut3, tut4, tut5, tut6, tut7, accountId) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name upd_tutorials
UPDATE account_tutorial SET tut0 = ?, tut1 = ?, tut2 = ?, tut3 = ?, tut4 = ?, tut5 = ?, tut6 = ?, tut7 = ? WHERE accountId = ?;

-- :name del_tutorials
DELETE FROM account_tutorial WHERE accountId = ?;

-- >>>>> Instance saves

-- :name ins_instance_save
INSERT INTO instance (id, map, resettime, difficulty, completedEncounters, data) VALUES (?, ?, ?, ?, ?, ?);

-- :name upd_instance_save_data
UPDATE instance SET data=? WHERE id=?;

-- :name upd_instance_save_encountermask
UPDATE instance SET completedEncounters=? WHERE id=?;

-- >>>>> Game event saves

-- :name del_game_event_save
DELETE FROM game_event_save WHERE eventEntry = ?;

-- :name ins_game_event_save
INSERT INTO game_event_save (eventEntry, state, next_start) VALUES (?, ?, ?);

-- >>>>> Game event condition saves

-- :name del_all_game_event_condition_save
DELETE FROM game_event_condition_save WHERE eventEntry = ?;

-- :name del_game_event_condition_save
DELETE FROM game_event_condition_save WHERE eventEntry = ? AND condition_id = ?;

-- :name ins_game_event_condition_save
INSERT INTO game_event_condition_save (eventEntry, condition_id, done) VALUES (?, ?, ?);

-- >>>>> Petitions

-- :name del_all_petition_signatures
DELETE FROM petition_sign WHERE playerguid = ?;

-- :name del_petition_signature
DELETE FROM petition_sign WHERE playerguid = ? AND type = ?;

-- >>>>> Arena teams

-- :name ins_arena_team
INSERT INTO arena_team (arenaTeamId, name, captainGuid, type, rating, backgroundColor, emblemStyle, emblemColor, borderStyle, borderColor) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name ins_arena_team_member
INSERT INTO arena_team_member (arenaTeamId, guid) VALUES (?, ?);

-- :name del_arena_team
DELETE FROM arena_team WHERE arenaTeamId = ?;

-- :name del_arena_team_members
DELETE FROM arena_team_member WHERE arenaTeamId = ?;

-- :name upd_arena_team_captain
UPDATE arena_team SET captainGuid = ? WHERE arenaTeamId = ?;

-- :name del_arena_team_member
DELETE FROM arena_team_member WHERE arenaTeamId = ? AND guid = ?;

-- :name upd_arena_team_stats
UPDATE arena_team SET rating = ?, weekGames = ?, weekWins = ?, seasonGames = ?, seasonWins = ?, `rank` = ? WHERE arenaTeamId = ?;

-- :name upd_arena_team_member
UPDATE arena_team_member SET personalRating = ?, weekGames = ?, weekWins = ?, seasonGames = ?, seasonWins = ? WHERE arenaTeamId = ? AND guid = ?;

-- :name rep_character_arena_stats
REPLACE INTO character_arena_stats (guid, slot, matchMakerRating, maxMMR) VALUES (?, ?, ?, ?);

-- :name sel_player_arena_teams
SELECT arena_team_member.arenaTeamId FROM arena_team_member JOIN arena_team ON arena_team_member.arenaTeamId = arena_team.arenaTeamId WHERE guid = ?;

-- :name upd_arena_team_name
UPDATE arena_team SET name = ? WHERE arenaTeamId = ?;

-- >>>>> Character battleground data

-- :name ins_player_entry_point
INSERT INTO character_entry_point (guid, joinX, joinY, joinZ, joinO, joinMapId, taxiPath0, taxiPath1, mountSpell) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_player_entry_point
DELETE FROM character_entry_point WHERE guid = ?;

-- >>>>> Character homebind

-- :name ins_player_homebind
INSERT INTO character_homebind (guid, mapId, zoneId, posX, posY, posZ, posO) VALUES (?, ?, ?, ?, ?, ?, ?);

-- :name upd_player_homebind
UPDATE character_homebind SET mapId = ?, zoneId = ?, posX = ?, posY = ?, posZ = ?, posO = ? WHERE guid = ?;

-- :name del_player_homebind
DELETE FROM character_homebind WHERE guid = ?;

-- >>>>> Corpse

-- :name sel_corpses
SELECT posX, posY, posZ, orientation, mapId, displayId, itemCache, bytes1, bytes2, guildId, flags, dynFlags, time, corpseType, instanceId, phaseMask, guid FROM corpse WHERE mapId = ? AND instanceId = ?;

-- :name ins_corpse
INSERT INTO corpse (guid, posX, posY, posZ, orientation, mapId, displayId, itemCache, bytes1, bytes2, guildId, flags, dynFlags, time, corpseType, instanceId, phaseMask) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_corpse
DELETE FROM corpse WHERE guid = ?;

-- :name del_corpses_from_map
DELETE FROM corpse WHERE mapId = ? AND instanceId = ?;

-- :name sel_corpse_location
SELECT mapId, posX, posY, posZ, orientation FROM corpse WHERE guid = ?;

-- >>>>> Creature respawn

-- :name sel_creature_respawns
SELECT guid, respawnTime FROM creature_respawn WHERE mapId = ? AND instanceId = ?;

-- :name rep_creature_respawn
REPLACE INTO creature_respawn (guid, respawnTime, mapId, instanceId) VALUES (?, ?, ?, ?);

-- :name del_creature_respawn
DELETE FROM creature_respawn WHERE guid = ? AND mapId = ? AND instanceId = ?;

-- :name del_creature_respawn_by_instance
DELETE FROM creature_respawn WHERE mapId = ? AND instanceId = ?;

-- >>>>> Gameobject respawn

-- :name sel_go_respawns
SELECT guid, respawnTime FROM gameobject_respawn WHERE mapId = ? AND instanceId = ?;

-- :name rep_go_respawn
REPLACE INTO gameobject_respawn (guid, respawnTime, mapId, instanceId) VALUES (?, ?, ?, ?);

-- :name del_go_respawn
DELETE FROM gameobject_respawn WHERE guid = ? AND mapId = ? AND instanceId = ?;

-- :name del_go_respawn_by_instance
DELETE FROM gameobject_respawn WHERE mapId = ? AND instanceId = ?;

-- >>>>> GM Tickets

-- :name sel_gm_tickets
SELECT id, type, playerGuid, name, description, createTime, mapId, posX, posY, posZ, lastModifiedTime, closedBy, assignedTo, comment, response, completed, escalated, viewed, needMoreHelp, resolvedBy FROM gm_ticket;

-- :name rep_gm_ticket
REPLACE INTO gm_ticket (id, type, playerGuid, name, description, createTime, mapId, posX, posY, posZ, lastModifiedTime, closedBy, assignedTo, comment, response, completed, escalated, viewed, needMoreHelp, resolvedBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_gm_ticket
DELETE FROM gm_ticket WHERE id = ?;

-- :name del_player_gm_tickets
DELETE FROM gm_ticket WHERE playerGuid = ?;

-- :name upd_player_gm_tickets_on_char_deletion
UPDATE gm_ticket SET type = 2 WHERE playerGuid = ?;

-- >>>>> GM Survey/subsurvey/lag report

-- :name ins_gm_survey
INSERT INTO gm_survey (guid, surveyId, mainSurvey, comment, createTime) VALUES (?, ?, ?, ?, UNIX_TIMESTAMP(NOW()));

-- :name ins_gm_subsurvey
INSERT INTO gm_subsurvey (surveyId, questionId, answer, answerComment) VALUES (?, ?, ?, ?);

-- :name ins_lag_report
INSERT INTO lag_reports (guid, lagType, mapId, posX, posY, posZ, latency, createTime) VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- >>>>> LFG Data

-- :name rep_lfg_data
REPLACE INTO lfg_data (guid, dungeon, state) VALUES (?, ?, ?);

-- :name del_lfg_data
DELETE FROM lfg_data WHERE guid = ?;

-- >>>>> Player saving

-- :name ins_character
INSERT INTO characters (guid, account, name, race, class, gender, level, xp, money, skin, face, hairStyle, hairColor, facialStyle, bankSlots, restState, playerFlags, 
                  map, instance_id, instance_mode_mask, position_x, position_y, position_z, orientation, trans_x, trans_y, trans_z, trans_o, transguid, 
                  taximask, cinematic, 
                  totaltime, leveltime, rest_bonus, logout_time, is_logout_resting, resettalents_cost, resettalents_time, 
                  extra_flags, stable_slots, at_login, zone, 
                  death_expire_time, taxi_path, arenaPoints, totalHonorPoints, todayHonorPoints, yesterdayHonorPoints, totalKills, 
                  todayKills, yesterdayKills, chosenTitle, knownCurrencies, watchedFaction, drunk, health, power1, power2, power3, 
                  power4, power5, power6, power7, latency, talentGroupsCount, activeTalentGroup, exploredZones, equipmentCache, 
                  ammoId, knownTitles, actionBars, grantableLevels, innTriggerId) VALUES 
                  (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?);

-- :name upd_character
UPDATE characters SET name=?,race=?,class=?,gender=?,level=?,xp=?,money=?,skin=?,face=?,hairStyle=?,hairColor=?,facialStyle=?,bankSlots=?,restState=?,playerFlags=?,
                  map=?,instance_id=?,instance_mode_mask=?,position_x=?,position_y=?,position_z=?,orientation=?,trans_x=?,trans_y=?,trans_z=?,trans_o=?,transguid=?,taximask=?,cinematic=?,totaltime=?,leveltime=?,rest_bonus=?,
                  logout_time=?,is_logout_resting=?,resettalents_cost=?,resettalents_time=?,extra_flags=?,stable_slots=?,at_login=?,zone=?,death_expire_time=?,taxi_path=?,
                  arenaPoints=?,totalHonorPoints=?,todayHonorPoints=?,yesterdayHonorPoints=?,totalKills=?,todayKills=?,yesterdayKills=?,chosenTitle=?,knownCurrencies=?,
                  watchedFaction=?,drunk=?,health=?,power1=?,power2=?,power3=?,power4=?,power5=?,power6=?,power7=?,latency=?,talentGroupsCount=?,activeTalentGroup=?,exploredZones=?,
                  equipmentCache=?,ammoId=?,knownTitles=?,actionBars=?,grantableLevels=?,innTriggerId=?,online=? WHERE guid=?;


-- :name upd_add_at_login_flag
UPDATE characters SET at_login = at_login | ? WHERE guid = ?;

-- :name upd_rem_at_login_flag
UPDATE characters set at_login = at_login & ~ ? WHERE guid = ?;

-- :name upd_all_at_login_flags
UPDATE characters SET at_login = at_login | ?;

-- :name ins_bug_report
INSERT INTO bugreport (type, content) VALUES(?, ?);

-- :name upd_petition_name
UPDATE petition SET name = ? WHERE petitionguid = ?;

-- :name ins_petition_signature
INSERT INTO petition_sign (ownerguid, petitionguid, playerguid, player_account) VALUES (?, ?, ?, ?);

-- :name upd_account_online
UPDATE characters SET online = 0 WHERE account = ?;

-- :name ins_group
INSERT INTO `groups` (guid, leaderGuid, lootMethod, looterGuid, lootThreshold, icon1, icon2, icon3, icon4, icon5, icon6, icon7, icon8, groupType, difficulty, raidDifficulty, masterLooterGuid) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name rep_group_member
REPLACE INTO group_member (guid, memberGuid, memberFlags, subgroup, roles) VALUES(?, ?, ?, ?, ?);

-- :name del_group_member
DELETE FROM group_member WHERE memberGuid = ? AND guid = ?;

-- :name upd_group_leader
UPDATE `groups` SET leaderGuid = ? WHERE guid = ?;

-- :name upd_group_type
UPDATE `groups` SET groupType = ? WHERE guid = ?;

-- :name upd_group_member_subgroup
UPDATE group_member SET subgroup = ? WHERE memberGuid = ?;

-- :name upd_group_member_flag
UPDATE group_member SET memberFlags = ? WHERE memberGuid = ?;

-- :name upd_group_difficulty
UPDATE `groups` SET difficulty = ? WHERE guid = ?;

-- :name upd_group_raid_difficulty
UPDATE `groups` SET raidDifficulty = ? WHERE guid = ?;

-- :name del_all_gm_tickets
TRUNCATE TABLE gm_ticket;

-- :name del_invalid_spell_talents
DELETE FROM character_talent WHERE spell = ?;

-- :name del_invalid_spell_spells
DELETE FROM character_spell WHERE spell = ?;

-- :name upd_delete_info
UPDATE characters SET deleteInfos_Name = name, deleteInfos_Account = account, deleteDate = UNIX_TIMESTAMP(), name = '', account = 0 WHERE guid = ?;

-- :name udp_restore_delete_info
UPDATE characters SET name = ?, account = ?, deleteDate = NULL, deleteInfos_Name = NULL, deleteInfos_Account = NULL WHERE deleteDate IS NOT NULL AND guid = ?;

-- :name upd_zone
UPDATE characters SET zone = ? WHERE guid = ?;

-- :name upd_level
UPDATE characters SET level = ?, xp = 0 WHERE guid = ?;

-- :name upd_xp_accumulative
UPDATE characters SET  xp = xp + ? WHERE guid = ?;

-- :name del_invalid_achiev_progress_criteria
DELETE FROM character_achievement_progress WHERE criteria = ?;

-- :name del_invalid_achievment
DELETE FROM character_achievement WHERE achievement = ?;

-- :name ins_addon
INSERT INTO addons (name, crc) VALUES (?, ?);

-- :name del_invalid_pet_spell
DELETE FROM pet_spell WHERE spell = ?;

-- :name upd_global_instance_resettime
UPDATE instance_reset SET resettime = ? WHERE mapid = ? AND difficulty = ?;

-- :name upd_char_online
UPDATE characters SET online = 1 WHERE guid = ?;

-- :name upd_char_name_at_login
UPDATE characters set name = ?, at_login = ? WHERE guid = ?;

-- :name upd_worldstate
UPDATE worldstates SET value = ? WHERE entry = ?;

-- :name ins_worldstate
INSERT INTO worldstates (entry, value) VALUES (?, ?);

-- :name del_char_instance_by_instance
DELETE FROM character_instance WHERE instance = ?;

-- :name del_char_instance_by_instance_not_extended
DELETE FROM character_instance WHERE instance = ? AND extended = 0;

-- :name upd_char_instance_set_not_extended
UPDATE character_instance SET extended = 0 WHERE instance = ?;

-- :name del_char_instance_by_instance_guid
DELETE FROM character_instance WHERE guid = ? AND instance = ?;

-- :name upd_char_instance
UPDATE character_instance SET instance = ?, permanent = ?, extended = 0 WHERE guid = ? AND instance = ?;

-- :name upd_char_instance_extended
UPDATE character_instance SET extended = ? WHERE guid = ? AND instance = ?;

-- :name ins_char_instance
INSERT INTO character_instance (guid, instance, permanent, extended) VALUES (?, ?, ?, 0);

-- :name ins_arena_log_fight
INSERT INTO log_arena_fights VALUES (?, NOW(), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name ins_arena_log_memberstats
INSERT INTO log_arena_memberstats VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name upd_gender_and_appearance
UPDATE characters SET gender = ?, skin = ?, face = ?, hairStyle = ?, hairColor = ?, facialStyle = ? WHERE guid = ?;

-- :name del_character_skill
DELETE FROM character_skills WHERE guid = ? AND skill = ?;

-- :name upd_add_character_social_flags
UPDATE character_social SET flags = flags | ? WHERE guid = ? AND friend = ?;

-- :name upd_rem_character_social_flags
UPDATE character_social SET flags = flags & ~ ? WHERE guid = ? AND friend = ?;

-- :name ins_character_social
REPLACE INTO character_social (guid, friend, flags) VALUES (?, ?, ?);

-- :name del_character_social
DELETE FROM character_social WHERE guid = ? AND friend = ?;

-- :name upd_character_social_note
UPDATE character_social SET note = ? WHERE guid = ? AND friend = ?;

-- :name upd_character_position
UPDATE characters SET position_x = ?, position_y = ?, position_z = ?, orientation = ?, map = ?, zone = ?, trans_x = 0, trans_y = 0, trans_z = 0, transguid = 0, taxi_path = '', cinematic = 1 WHERE guid = ?;

-- :name sel_character_aura_frozen
SELECT characters.name FROM characters LEFT JOIN character_aura ON (characters.guid = character_aura.guid) WHERE character_aura.spell = 9454;

-- :name sel_character_online
SELECT name, account, map, zone FROM characters WHERE online > 0;

-- :name sel_char_del_info_by_guid
SELECT guid, deleteInfos_Name, deleteInfos_Account, deleteDate FROM characters WHERE deleteDate IS NOT NULL AND guid = ?;

-- :name sel_char_del_info_by_name
SELECT guid, deleteInfos_Name, deleteInfos_Account, deleteDate FROM characters WHERE deleteDate IS NOT NULL AND deleteInfos_Name LIKE CONCAT('%%', ?, '%%');

-- :name sel_char_del_info
SELECT guid, deleteInfos_Name, deleteInfos_Account, deleteDate FROM characters WHERE deleteDate IS NOT NULL;

-- :name sel_chars_by_account_id
SELECT guid FROM characters WHERE account = ?;

-- :name sel_char_pinfo
SELECT totaltime, level, money, account, race, class, map, zone, gender, health, playerFlags FROM characters WHERE guid = ?;

-- :name sel_pinfo_bans
SELECT unbandate, bandate = unbandate, bannedby, banreason FROM character_banned WHERE guid = ? AND active ORDER BY bandate ASC LIMIT 1;

-- :name sel_pinfo_mails
SELECT SUM(CASE WHEN (checked & 1) THEN 1 ELSE 0 END) AS 'readmail', COUNT(*) AS 'totalmail' FROM mail WHERE `receiver` = ?;

-- :name sel_pinfo_xp
SELECT a.xp, b.guid FROM characters a LEFT JOIN guild_member b ON a.guid = b.guid WHERE a.guid = ?;

-- :name sel_char_homebind
SELECT mapId, zoneId, posX, posY, posZ, posO FROM character_homebind WHERE guid = ?;

-- :name sel_char_guid_name_by_acc
SELECT guid, name FROM characters WHERE account = ?;

-- :name sel_pool_quest_save
SELECT quest_id FROM pool_quest_save WHERE pool_id = ?;

-- :name sel_character_at_login
SELECT at_login FROM characters WHERE guid = ?;

-- :name sel_char_class_lvl_at_login
SELECT class, level, at_login, knownTitles FROM characters WHERE guid = ?;

-- :name sel_char_customize_info
SELECT name, race, class, gender, at_login FROM characters WHERE guid = ?;

-- :name sel_char_race_or_faction_change_infos
SELECT at_login, knownTitles, money FROM characters WHERE guid = ?;

-- :name sel_char_at_login_titles_money
SELECT at_login, knownTitles, money FROM characters WHERE guid = ?;

-- :name sel_char_cod_item_mail
SELECT id, messageType, mailTemplateId, sender, subject, body, money, has_items FROM mail WHERE receiver = ? AND has_items <> 0 AND cod <> 0;

-- :name sel_char_social
SELECT DISTINCT guid FROM character_social WHERE friend = ?;

-- :name sel_char_old_chars
SELECT guid, deleteInfos_Account FROM characters WHERE deleteDate IS NOT NULL AND deleteDate < ?;

-- :name sel_arena_team_id_by_player_guid
SELECT arena_team_member.arenateamid FROM arena_team_member JOIN arena_team ON arena_team_member.arenateamid = arena_team.arenateamid WHERE guid = ? AND type = ? LIMIT 1;

-- :name sel_mail
SELECT id, messageType, sender, receiver, subject, body, expire_time, deliver_time, money, cod, checked, stationery, mailTemplateId FROM mail WHERE receiver = ? AND deliver_time <= ? ORDER BY id DESC;

-- :name sel_next_mail_deliverytime
SELECT MIN(deliver_time) FROM mail WHERE receiver = ? AND deliver_time > ? AND (checked & 1) = 0 LIMIT 1;

-- :name del_char_aura_frozen
DELETE FROM character_aura WHERE spell = 9454 AND guid = ?;

-- :name sel_char_inventory_count_item
SELECT COUNT(itemEntry) FROM character_inventory ci INNER JOIN item_instance ii ON ii.guid = ci.item WHERE itemEntry = ?;

-- :name sel_mail_count_item
SELECT COUNT(itemEntry) FROM mail_items mi INNER JOIN item_instance ii ON ii.guid = mi.item_guid WHERE itemEntry = ?;

-- :name sel_auctionhouse_count_item
SELECT COUNT(itemEntry) FROM auctionhouse ah INNER JOIN item_instance ii ON ii.guid = ah.itemguid WHERE itemEntry = ?;

-- :name sel_guild_bank_count_item
SELECT COUNT(itemEntry) FROM guild_bank_item gbi INNER JOIN item_instance ii ON ii.guid = gbi.item_guid WHERE itemEntry = ?;

-- :name sel_char_inventory_item_by_entry
SELECT ci.item, cb.slot AS bag, ci.slot, ci.guid, c.account, c.name FROM characters c 
                  INNER JOIN character_inventory ci ON ci.guid = c.guid 
                  INNER JOIN item_instance ii ON ii.guid = ci.item 
                  LEFT JOIN character_inventory cb ON cb.item = ci.bag WHERE ii.itemEntry = ? LIMIT ?;

-- :name sel_char_inventory_item_by_entry_and_owner
SELECT ci.item FROM character_inventory ci INNER JOIN item_instance ii ON ii.guid = ci.item WHERE ii.itemEntry = ? AND ii.owner_guid = ?;

-- :name sel_mail_items_by_entry
SELECT mi.item_guid, m.sender, m.receiver, cs.account, cs.name, cr.account, cr.name 
                  FROM mail m INNER JOIN mail_items mi ON mi.mail_id = m.id INNER JOIN item_instance ii ON ii.guid = mi.item_guid 
                  INNER JOIN characters cs ON cs.guid = m.sender INNER JOIN characters cr ON cr.guid = m.receiver WHERE ii.itemEntry = ? LIMIT ?;

-- :name sel_auctionhouse_item_by_entry
SELECT  ah.itemguid, ah.itemowner, c.account, c.name FROM auctionhouse ah INNER JOIN characters c ON c.guid = ah.itemowner INNER JOIN item_instance ii ON ii.guid = ah.itemguid WHERE ii.itemEntry = ? LIMIT ?;

-- :name sel_guild_bank_item_by_entry
SELECT gi.item_guid, gi.guildid, g.name FROM guild_bank_item gi INNER JOIN guild g ON g.guildid = gi.guildid INNER JOIN item_instance ii ON ii.guid = gi.item_guid WHERE ii.itemEntry = ? LIMIT ?;

-- :name del_char_achievement
DELETE FROM character_achievement WHERE guid = ?;

-- :name del_char_achievement_progress
DELETE FROM character_achievement_progress WHERE guid = ?;

-- :name ins_char_achievement
INSERT INTO character_achievement (guid, achievement, date) VALUES (?, ?, ?);

-- :name del_char_achievement_progress_by_criteria
DELETE FROM character_achievement_progress WHERE guid = ? AND criteria = ?;

-- :name ins_char_achievement_progress
INSERT INTO character_achievement_progress (guid, criteria, counter, date) VALUES (?, ?, ?, ?);

-- :name del_char_reputation_by_faction
DELETE FROM character_reputation WHERE guid = ? AND faction = ?;

-- :name ins_char_reputation_by_faction
INSERT INTO character_reputation (guid, faction, standing, flags) VALUES (?, ?, ? , ?);

-- :name upd_char_arena_points
UPDATE characters SET arenaPoints = (arenaPoints + ?) WHERE guid = ?;

-- :name del_item_refund_instance
DELETE FROM item_refund_instance WHERE item_guid = ?;

-- :name ins_item_refund_instance
INSERT INTO item_refund_instance (item_guid, player_guid, paidMoney, paidExtendedCost) VALUES (?, ?, ?, ?);

-- :name del_group
DELETE FROM `groups` WHERE guid = ?;

-- :name del_group_member_all
DELETE FROM group_member WHERE guid = ?;

-- :name ins_char_gift
INSERT INTO character_gifts (guid, item_guid, entry, flags) VALUES (?, ?, ?, ?);

-- :name del_instance_by_instance
DELETE FROM instance WHERE id = ?;

-- :name del_mail_item_by_id
DELETE FROM mail_items WHERE mail_id = ?;

-- :name ins_petition
INSERT INTO petition (ownerguid, petitionguid, name, type) VALUES (?, ?, ?, ?);

-- :name del_petition_by_guid
DELETE FROM petition WHERE petitionguid = ?;

-- :name del_petition_signature_by_guid
DELETE FROM petition_sign WHERE petitionguid = ?;

-- :name del_char_declined_name
DELETE FROM character_declinedname WHERE guid = ?;

-- :name ins_char_declined_name
INSERT INTO character_declinedname (guid, genitive, dative, accusative, instrumental, prepositional) VALUES (?, ?, ?, ?, ?, ?);

-- :name upd_char_race
UPDATE characters SET race = ? WHERE guid = ?;

-- :name del_char_skill_languages
DELETE FROM character_skills WHERE skill IN (98, 113, 759, 111, 313, 109, 115, 315, 673, 137) AND guid = ?;

-- :name ins_char_skill_language
INSERT INTO `character_skills` (guid, skill, value, max) VALUES (?, ?, 300, 300);

-- :name upd_char_taxi_path
UPDATE characters SET taxi_path = '' WHERE guid = ?;

-- :name upd_char_taximask
UPDATE characters SET taximask = ? WHERE guid = ?;

-- :name del_char_queststatus
DELETE FROM character_queststatus WHERE guid = ?;

-- :name del_char_social_by_guid
DELETE FROM character_social WHERE guid = ?;

-- :name del_char_social_by_friend
DELETE FROM character_social WHERE friend = ?;

-- :name del_char_achievement_by_achievement
DELETE FROM character_achievement WHERE achievement = ? AND guid = ?;

-- :name upd_char_achievement
UPDATE character_achievement SET achievement = ? WHERE achievement = ? AND guid = ?;

-- :name upd_char_inventory_faction_change
UPDATE item_instance ii, character_inventory ci SET ii.itemEntry = ? WHERE ii.itemEntry = ? AND ci.guid = ? AND ci.item = ii.guid;

-- :name del_char_spell_by_spell
DELETE FROM character_spell WHERE guid = ? AND spell = ?;

-- :name upd_char_spell_faction_change
UPDATE character_spell SET spell = ? WHERE spell = ? AND guid = ?;

-- :name sel_char_rep_by_faction
SELECT standing FROM character_reputation WHERE faction = ? AND guid = ?;

-- :name del_char_rep_by_faction
DELETE FROM character_reputation WHERE faction = ? AND guid = ?;

-- :name upd_char_rep_faction_change
UPDATE character_reputation SET faction = ?, standing = ? WHERE faction = ? AND guid = ?;

-- :name upd_char_titles_faction_change
UPDATE characters SET knownTitles = ? WHERE guid = ?;

-- :name res_char_titles_faction_change
UPDATE characters SET chosenTitle = 0 WHERE guid = ?;

-- :name del_char_spell_cooldown
DELETE FROM character_spell_cooldown WHERE guid = ?;

-- :name del_character
DELETE FROM characters WHERE guid = ?;

-- :name del_char_action
DELETE FROM character_action WHERE guid = ?;

-- :name del_char_aura
DELETE FROM character_aura WHERE guid = ?;

-- :name del_char_gift
DELETE FROM character_gifts WHERE guid = ?;

-- :name del_char_instance
DELETE FROM character_instance WHERE guid = ?;

-- :name del_char_inventory
DELETE FROM character_inventory WHERE guid = ?;

-- :name del_char_queststatus_rewarded
DELETE FROM character_queststatus_rewarded WHERE guid = ?;

-- :name del_char_reputation
DELETE FROM character_reputation WHERE guid = ?;

-- :name del_char_spell
DELETE FROM character_spell WHERE guid = ?;

-- :name del_mail
DELETE FROM mail WHERE receiver = ?;

-- :name del_mail_items
DELETE FROM mail_items WHERE receiver = ?;

-- :name del_char_achievements
DELETE FROM character_achievement WHERE guid = ? AND achievement NOT BETWEEN '456' AND '467' AND achievement NOT BETWEEN '1400' AND '1427' AND achievement NOT IN(1463, 3117, 3259);

-- :name del_char_equipmentsets
DELETE FROM character_equipmentsets WHERE guid = ?;

-- :name del_guild_eventlog_by_player
DELETE FROM guild_eventlog WHERE PlayerGuid1 = ? OR PlayerGuid2 = ?;

-- :name del_guild_bank_eventlog_by_player
DELETE FROM guild_bank_eventlog WHERE PlayerGuid = ?;

-- :name del_char_glyphs
DELETE FROM character_glyphs WHERE guid = ?;

-- :name del_char_talent
DELETE FROM character_talent WHERE guid = ?;

-- :name del_char_skills
DELETE FROM character_skills WHERE guid = ?;

-- :name udp_char_honor_points
UPDATE characters SET totalHonorPoints = ? WHERE guid = ?;

-- :name udp_char_honor_points_accumulative
UPDATE characters SET totalHonorPoints = totalHonorPoints + ? WHERE guid = ?;

-- :name udp_char_arena_points
UPDATE characters SET arenaPoints = ? WHERE guid = ?;

-- :name udp_char_arena_points_accumulative
UPDATE characters SET arenaPoints = arenaPoints + ? WHERE guid = ?;

-- :name udp_char_money
UPDATE characters SET money = ? WHERE guid = ?;

-- :name udp_char_money_accumulative
UPDATE characters SET money = money + ? WHERE guid = ?;

-- :name upd_char_remove_ghost
UPDATE characters SET playerFlags = (playerFlags & (~16)) WHERE guid = ?;

-- :name ins_char_action
INSERT INTO character_action (guid, spec, button, action, type) VALUES (?, ?, ?, ?, ?);

-- :name upd_char_action
UPDATE character_action SET action = ?, type = ? WHERE guid = ? AND button = ? AND spec = ?;

-- :name del_char_action_by_button_spec
DELETE FROM character_action WHERE guid = ? AND button = ? AND spec = ?;

-- :name del_char_inventory_by_item
DELETE FROM character_inventory WHERE item = ?;

-- :name del_char_inventory_by_bag_slot
DELETE FROM character_inventory WHERE bag = ? AND slot = ? AND guid = ?;

-- :name upd_mail
UPDATE mail SET has_items = ?, expire_time = ?, deliver_time = ?, money = ?, cod = ?, checked = ? WHERE id = ?;

-- :name rep_char_queststatus
REPLACE INTO character_queststatus (guid, quest, status, explored, timer, mobcount1, mobcount2, mobcount3, mobcount4, itemcount1, itemcount2, itemcount3, itemcount4, itemcount5, itemcount6, playercount) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_char_queststatus_by_quest
DELETE FROM character_queststatus WHERE guid = ? AND quest = ?;

-- :name ins_char_queststatus_rewarded
INSERT IGNORE INTO character_queststatus_rewarded (guid, quest, active) VALUES (?, ?, 1);

-- :name del_char_queststatus_rewarded_by_quest
DELETE FROM character_queststatus_rewarded WHERE guid = ? AND quest = ?;

-- :name upd_char_queststatus_rewarded_faction_change
UPDATE character_queststatus_rewarded SET quest = ? WHERE quest = ? AND guid = ?;

-- :name upd_char_queststatus_rewarded_active
UPDATE character_queststatus_rewarded SET active = 1 WHERE guid = ?;

-- :name upd_char_queststatus_rewarded_active_by_quest
UPDATE character_queststatus_rewarded SET active = 0 WHERE quest = ? AND guid = ?;

-- :name del_char_skill_by_skill
DELETE FROM character_skills WHERE guid = ? AND skill = ?;

-- :name ins_char_skills
INSERT INTO character_skills (guid, skill, value, max) VALUES (?, ?, ?, ?);

-- :name udp_char_skills
UPDATE character_skills SET value = ?, max = ? WHERE guid = ? AND skill = ?;

-- :name ins_char_spell
INSERT INTO character_spell (guid, spell, specMask) VALUES (?, ?, ?);

-- :name del_char_stats
DELETE FROM character_stats WHERE guid = ?;

-- :name ins_char_stats
INSERT INTO character_stats (guid, maxhealth, maxpower1, maxpower2, maxpower3, maxpower4, maxpower5, maxpower6, maxpower7, strength, agility, stamina, intellect, spirit, 
                  armor, resHoly, resFire, resNature, resFrost, resShadow, resArcane, blockPct, dodgePct, parryPct, critPct, rangedCritPct, spellCritPct, attackPower, rangedAttackPower, 
                  spellPower, resilience) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name sel_char_stats
SELECT maxhealth, strength, agility, stamina, intellect, spirit, armor, attackPower, spellPower, resilience FROM character_stats WHERE guid = ?;

-- :name del_petition_by_owner
DELETE FROM petition WHERE ownerguid = ?;

-- :name del_petition_signature_by_owner
DELETE FROM petition_sign WHERE ownerguid = ?;

-- :name del_petition_by_owner_and_type
DELETE FROM petition WHERE ownerguid = ? AND type = ?;

-- :name del_petition_signature_by_owner_and_type
DELETE FROM petition_sign WHERE ownerguid = ? AND type = ?;

-- :name ins_char_glyphs
INSERT INTO character_glyphs VALUES(?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_char_talent_by_spell
DELETE FROM character_talent WHERE guid = ? AND spell = ?;

-- :name ins_char_talent
INSERT INTO character_talent (guid, spell, specMask) VALUES (?, ?, ?);

-- :name del_char_action_except_spec
DELETE FROM character_action WHERE spec<>? AND guid = ?;

-- >>>>> Items that hold loot or money

-- :name sel_itemcontainer_items
SELECT containerGUID, itemid, count, item_index, randomPropertyId, randomSuffix, follow_loot_rules, freeforall, is_blocked, is_counted, is_underthreshold, needs_quest, conditionLootId FROM item_loot_storage;

-- :name del_itemcontainer_single_item
DELETE FROM item_loot_storage WHERE containerGUID = ? AND itemid = ? AND count = ? AND item_index = ? LIMIT 1;

-- :name ins_itemcontainer_single_item
INSERT INTO item_loot_storage (containerGUID, itemid, item_index, count, randomPropertyId, randomSuffix, follow_loot_rules, freeforall, is_blocked, is_counted, is_underthreshold, needs_quest, conditionLootId) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_itemcontainer_container
DELETE FROM item_loot_storage WHERE containerGUID = ?;

-- >>>>> Calendar

-- :name rep_calendar_event
REPLACE INTO calendar_events (id, creator, title, description, type, dungeon, eventtime, flags, time2) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_calendar_event
DELETE FROM calendar_events WHERE id = ?;

-- :name rep_calendar_invite
REPLACE INTO calendar_invites (id, event, invitee, sender, status, statustime, `rank`, text) VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_calendar_invite
DELETE FROM calendar_invites WHERE id = ?;

-- >>>>> Pet

-- :name sel_char_pet_ids
SELECT id FROM character_pet WHERE owner = ?;

-- :name del_char_pet_declinedname_by_owner
DELETE FROM character_pet_declinedname WHERE owner = ?;

-- :name del_char_pet_declinedname
DELETE FROM character_pet_declinedname WHERE id = ?;

-- :name add_char_pet_declinedname
INSERT INTO character_pet_declinedname (id, owner, genitive, dative, accusative, instrumental, prepositional) VALUES (?, ?, ?, ?, ?, ?, ?);

-- :name sel_pet_declined_name
SELECT genitive, dative, accusative, instrumental, prepositional FROM character_pet_declinedname WHERE owner = ? AND id = ?;

-- :name sel_pet_aura
SELECT casterGuid, spell, effectMask, recalculateMask, stackCount, amount0, amount1, amount2, base_amount0, base_amount1, base_amount2, maxDuration, remainTime, remainCharges FROM pet_aura WHERE guid = ?;

-- :name sel_pet_spell
SELECT spell, active FROM pet_spell WHERE guid = ?;

-- :name sel_pet_spell_cooldown
SELECT spell, category, time FROM pet_spell_cooldown WHERE guid = ?;

-- :name del_pet_auras
DELETE FROM pet_aura WHERE guid = ?;

-- :name del_pet_spells
DELETE FROM pet_spell WHERE guid = ?;

-- :name del_pet_spell_cooldowns
DELETE FROM pet_spell_cooldown WHERE guid = ?;

-- :name ins_pet_spell_cooldown
INSERT INTO pet_spell_cooldown (guid, spell, category, time) VALUES (?, ?, ?, ?);

-- :name del_pet_spell_by_spell
DELETE FROM pet_spell WHERE guid = ? AND spell = ?;

-- :name ins_pet_spell
INSERT INTO pet_spell (guid, spell, active) VALUES (?, ?, ?);

-- :name ins_pet_aura
INSERT INTO pet_aura (guid, casterGuid, spell, effectMask, recalculateMask, stackCount, amount0, amount1, amount2, 
                  base_amount0, base_amount1, base_amount2, maxDuration, remainTime, remainCharges) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name sel_char_pets
SELECT id, entry, modelid, level, exp, Reactstate, slot, name, renamed, curhealth, curmana, curhappiness, abdata, savetime, CreatedBySpell, PetType FROM character_pet WHERE owner = ?;

-- :name del_char_pet_by_owner
DELETE FROM character_pet WHERE owner = ?;

-- :name upd_char_pet_name
UPDATE character_pet SET name = ?, renamed = 1 WHERE owner = ? AND id = ?;

-- :name upd_char_pet_slot_by_id
UPDATE character_pet SET slot = ? WHERE owner = ? AND id = ?;

-- :name del_char_pet_by_id
DELETE FROM character_pet WHERE id = ?;

-- :name del_char_pet_by_slot
DELETE FROM character_pet WHERE owner = ? AND (slot = ? OR slot > ?);

-- :name rep_char_pet
REPLACE INTO character_pet (id, entry, owner, modelid, CreatedBySpell, PetType, level, exp, Reactstate, name, renamed, slot, curhealth, curmana, curhappiness, savetime, abdata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- >>>>> PvPstats

-- :name sel_pvpstats_maxid
SELECT MAX(id) FROM pvpstats_battlegrounds;

-- :name ins_pvpstats_battleground
INSERT INTO pvpstats_battlegrounds (id, winner_faction, bracket_id, type, date) VALUES (?, ?, ?, ?, NOW());

-- :name ins_pvpstats_player
INSERT INTO pvpstats_players (battleground_id, character_guid, winner, score_killing_blows, score_deaths, score_honorable_kills, score_bonus_honor, score_damage_done, score_healing_done, attr_1, attr_2, attr_3, attr_4, attr_5) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name sel_pvpstats_factions_overall
SELECT winner_faction, COUNT(*) AS count FROM pvpstats_battlegrounds WHERE DATEDIFF(NOW(), date) < 7 GROUP BY winner_faction ORDER BY winner_faction ASC;

-- :name sel_pvpstats_bracket_month
SELECT character_guid, COUNT(character_guid) AS count, characters.name as character_name FROM pvpstats_players INNER JOIN pvpstats_battlegrounds ON pvpstats_players.battleground_id = pvpstats_battlegrounds.id AND bracket_id = ? AND MONTH(date) = MONTH(NOW()) AND YEAR(date) = YEAR(NOW()) INNER JOIN characters ON pvpstats_players.character_guid = characters.guid AND characters.deleteDate IS NULL WHERE pvpstats_players.winner = 1 GROUP BY character_guid ORDER BY count(character_guid) DESC LIMIT 0, ?;

-- >>>>> Deserter tracker

-- :name ins_deserter_track
INSERT INTO battleground_deserters (guid, type, datetime) VALUES (?, ?, NOW());

-- >>>>> QuestTracker

-- :name ins_quest_track
INSERT INTO quest_tracker (id, character_guid, quest_accept_time, core_hash, core_revision) VALUES (?, ?, NOW(), ?, ?);

-- :name upd_quest_track_gm_complete
UPDATE quest_tracker SET completed_by_gm = 1 WHERE id = ? AND character_guid = ? ORDER BY quest_accept_time DESC LIMIT 1;

-- :name upd_quest_track_complete_time
UPDATE quest_tracker SET quest_complete_time = NOW() WHERE id = ? AND character_guid = ? ORDER BY quest_accept_time DESC LIMIT 1;

-- :name upd_quest_track_abandon_time
UPDATE quest_tracker SET quest_abandon_time = NOW() WHERE id = ? AND character_guid = ? ORDER BY quest_accept_time DESC LIMIT 1;

-- >>>>> Recovery Item

-- :name ins_recovery_item
INSERT INTO recovery_item (Guid, ItemEntry, Count) VALUES (?, ?, ?);

-- :name sel_recovery_item
SELECT id, itemEntry, Count, Guid FROM recovery_item WHERE id = ?;

-- :name sel_recovery_item_list
SELECT id, itemEntry, Count FROM recovery_item WHERE Guid = ? ORDER BY id DESC;

-- :name del_recovery_item
DELETE FROM recovery_item WHERE Guid = ? AND ItemEntry = ? AND Count = ? ORDER BY Id DESC LIMIT 1;

-- :name del_recovery_item_by_recovery_id
DELETE FROM recovery_item WHERE id = ?;


-- :name sel_honorpoints
SELECT totalHonorPoints FROM characters WHERE guid = ?;

-- :name sel_arenapoints
SELECT arenaPoints FROM characters WHERE guid = ?;

-- >>>>> Character names

-- :name ins_reserved_player_name
INSERT IGNORE INTO reserved_name (name) VALUES (?);

-- :name ins_profanity_player_name
INSERT IGNORE INTO profanity_name (name) VALUES (?);

-- >>>>> Character settings

-- :name sel_char_settings
SELECT source, data FROM character_settings WHERE guid = ?;

-- :name rep_char_settings
REPLACE INTO character_settings (guid, source, data) VALUES (?, ?, ?);

-- :name del_char_settings
DELETE FROM character_settings WHERE guid = ?;

-- >>>>> Instance saved data. Stores the states of gameobjects in instances to be loaded on server start

-- :name select_instance_saved_data
SELECT id, guid, state FROM instance_saved_go_state_data;

-- :name update_instance_saved_data
UPDATE instance_saved_go_state_data SET state = ? WHERE guid = ? AND id = ?;

-- :name insert_instance_saved_data
INSERT INTO instance_saved_go_state_data (id, guid, state) VALUES (?, ?, ?);

-- :name delete_instance_saved_data
DELETE FROM instance_saved_go_state_data WHERE id = ?;

-- :name sanitize_instance_saved_data
DELETE FROM instance_saved_go_state_data WHERE id NOT IN (SELECT instance.id FROM instance);