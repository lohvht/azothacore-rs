-- :name sel_realmlist :typed :^
-- :doc Returns all realms from DB ordered by name
SELECT id, name, address, localAddress, localSubnetMask, port, icon, flag, timezone, allowedSecurityLevel, population, gamebuild, Region, Battlegroup FROM realmlist WHERE flag <> 3 ORDER BY name;

-- :name del_expired_ip_bans
-- :doc Removes expired IP bans from DB
DELETE FROM ip_banned WHERE unbandate<>bandate AND unbandate<=UNIX_TIMESTAMP();

-- :name upd_expired_account_bans
UPDATE account_banned SET active = 0 WHERE active = 1 AND unbandate<>bandate AND unbandate<=UNIX_TIMESTAMP();

-- :name sel_ip_info :*
-- :doc Select IP Info. NOTE: TrinityCore's query is as below, we are not interested in ip2nation yet.
--      (SELECT unbandate > UNIX_TIMESTAMP() OR unbandate = bandate AS banned, NULL as country FROM ip_banned WHERE ip = ?)
--      UNION
--      (SELECT NULL AS banned, country FROM ip2nation WHERE INET_NTOA(ip) = ?);
SELECT unbandate > UNIX_TIMESTAMP() OR unbandate = bandate AS banned, NULL as country FROM ip_banned WHERE ip = ?;

-- :name ins_ip_auto_banned
INSERT INTO ip_banned (ip, bandate, unbandate, bannedby, banreason) VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, 'Azothacore Auth', 'Failed login autoban');

-- :name sel_ip_banned_all
SELECT ip, bandate, unbandate, bannedby, banreason FROM ip_banned WHERE (bandate = unbandate OR unbandate > UNIX_TIMESTAMP()) ORDER BY unbandate;

-- :name sel_ip_banned_by_ip
SELECT ip, bandate, unbandate, bannedby, banreason FROM ip_banned WHERE (bandate = unbandate OR unbandate > UNIX_TIMESTAMP()) AND ip LIKE CONCAT('%%', ?, '%%') ORDER BY unbandate;

-- :name sel_account_banned_all :typed :*
SELECT account.id, username FROM account, account_banned WHERE account.id = account_banned.id AND active = 1 GROUP BY account.id;

-- :name sel_account_banned_by_username :typed :*
SELECT account.id, username FROM account, account_banned WHERE account.id = account_banned.id AND active = 1 AND username LIKE CONCAT('%%', ?, '%%') GROUP BY account.id;

-- :name del_account_banned
DELETE FROM account_banned WHERE id = ?;

-- :name upd_account_info_continued_session
UPDATE account SET sessionkey = ? WHERE id = ?;

-- :name sel_account_info_continued_session
SELECT username, sessionkey FROM account WHERE id = ?;

-- :name upd_vs
UPDATE account SET v = ?, s = ? WHERE username = ?;

-- :name sel_account_id_by_name
SELECT id FROM account WHERE username = ?;

-- :name sel_account_list_by_name
SELECT id, username FROM account WHERE username = ?;

-- :name sel_account_info_by_name
SELECT a.id, a.sessionkey, ba.last_ip, ba.locked, ba.lock_country, a.expansion, a.mutetime, ba.locale, a.recruiter, a.os, ba.id, aa.gmLevel,
bab.unbandate > UNIX_TIMESTAMP() OR bab.unbandate = bab.bandate, ab.unbandate > UNIX_TIMESTAMP() OR ab.unbandate = ab.bandate, r.id
FROM account a LEFT JOIN account r ON a.id = r.recruiter LEFT JOIN battlenet_accounts ba ON a.battlenet_account = ba.id
LEFT JOIN account_access aa ON a.id = aa.id AND aa.RealmID IN (-1, ?) LEFT JOIN battlenet_account_bans bab ON ba.id = bab.id LEFT JOIN account_banned ab ON a.id = ab.id AND ab.active = 1
WHERE a.username = ? ORDER BY aa.RealmID DESC LIMIT 1;

-- :name sel_account_list_by_email
SELECT id, username FROM account WHERE email = ?;

-- :name sel_account_by_ip
SELECT id, username FROM account WHERE last_ip = ?;

-- :name sel_account_by_id :?
SELECT 1 FROM account WHERE id = ?;

-- :name ins_ip_banned
INSERT INTO ip_banned (ip, bandate, unbandate, bannedby, banreason) VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, ?, ?);

-- :name del_ip_not_banned
DELETE FROM ip_banned WHERE ip = ?;

-- :name ins_account_banned
INSERT INTO account_banned VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, ?, ?, 1);

-- :name upd_account_not_banned
UPDATE account_banned SET active = 0 WHERE id = ? AND active != 0;

-- :name del_realm_characters_by_realm
DELETE FROM realmcharacters WHERE acctid = ? AND realmid = ?;

-- :name del_realm_characters
DELETE FROM realmcharacters WHERE acctid = ?;

-- :name ins_realm_characters
INSERT INTO realmcharacters (numchars, acctid, realmid) VALUES (?, ?, ?);

-- :name sel_sum_realm_characters
SELECT SUM(numchars) FROM realmcharacters WHERE acctid = ?;

-- :name ins_account
INSERT INTO account(username, sha_pass_hash, reg_mail, email, joindate, battlenet_account, battlenet_index) VALUES(?, ?, ?, ?, NOW(), ?, ?);

-- :name ins_realm_characters_init
INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid=account.id WHERE acctid IS NULL;

-- :name upd_expansion
UPDATE account SET expansion = ? WHERE id = ?;

-- :name upd_account_lock
UPDATE account SET locked = ? WHERE id = ?;

-- :name upd_account_lock_country
UPDATE account SET lock_country = ? WHERE id = ?;

-- :name upd_username
UPDATE account SET v = 0, s = 0, username = ?, sha_pass_hash = ? WHERE id = ?;

-- :name upd_password
UPDATE account SET v = 0, s = 0, sha_pass_hash = ? WHERE id = ?;

-- :name upd_email
UPDATE account SET email = ? WHERE id = ?;

-- :name upd_reg_email
UPDATE account SET reg_mail = ? WHERE id = ?;

-- :name upd_mute_time
UPDATE account SET mutetime = ? , mutereason = ? , muteby = ? WHERE id = ?;

-- :name upd_mute_time_login
UPDATE account SET mutetime = ? WHERE id = ?;

-- :name upd_last_ip
UPDATE account SET last_ip = ? WHERE username = ?;

-- :name upd_last_attempt_ip
UPDATE account SET last_attempt_ip = ? WHERE username = ?;

-- :name upd_account_online
UPDATE account SET online = ? WHERE id = ?;

-- :name upd_uptime_players
UPDATE uptime SET uptime = ?, maxplayers = ? WHERE realmid = ? AND starttime = ?;

-- :name del_account_access
DELETE FROM account_access WHERE id = ?;

-- :name del_account_access_by_realm
DELETE FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1);

-- :name ins_account_access
INSERT INTO account_access (id,gmlevel,RealmID) VALUES (?, ?, ?);

-- :name get_account_id_by_username :typed :?
SELECT id FROM account WHERE username = ?;

-- :name get_account_access_gmlevel :typed :?
SELECT gmlevel FROM account_access WHERE id = ?;

-- :name get_gmlevel_by_realmid :typed :?
SELECT gmlevel FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1);

-- :name get_username_by_id :typed :?
SELECT username FROM account WHERE id = ?;

-- :name sel_check_password :?
SELECT 1 FROM account WHERE id = ? AND sha_pass_hash = ?;

-- :name sel_check_password_by_name
SELECT 1 FROM account WHERE username = ? AND sha_pass_hash = ?;

-- :name sel_pinfo
SELECT a.username, aa.gmlevel, a.email, a.reg_mail, a.last_ip, DATE_FORMAT(a.last_login, '%Y-%m-%d %T'), a.mutetime, a.mutereason, a.muteby, a.failed_logins, a.locked, a.OS FROM account a LEFT JOIN account_access aa ON (a.id = aa.id AND (aa.RealmID = ? OR aa.RealmID = -1)) WHERE a.id = ?;

-- :name sel_pinfo_bans
SELECT unbandate, bandate = unbandate, bannedby, banreason FROM account_banned WHERE id = ? AND active ORDER BY bandate ASC LIMIT 1;

-- :name sel_gm_accounts
SELECT a.username, aa.gmlevel FROM account a, account_access aa WHERE a.id=aa.id AND aa.gmlevel >= ? AND (aa.realmid = -1 OR aa.realmid = ?);

-- :name sel_account_info
-- :doc Only used in .account onlinelist command
SELECT a.username, a.last_ip, aa.gmlevel, a.expansion FROM account a LEFT JOIN account_access aa ON (a.id = aa.id) WHERE a.id = ? ORDER BY a.last_ip;

-- :name sel_account_access_gmlevel_test
SELECT 1 FROM account_access WHERE id = ? AND gmlevel > ?;

-- :name sel_account_access
SELECT a.id, aa.gmlevel, aa.RealmID FROM account a LEFT JOIN account_access aa ON (a.id = aa.id) WHERE a.username = ?;

-- :name sel_account_whois
SELECT username, email, last_ip FROM account WHERE id = ?;

-- :name sel_last_attempt_ip
SELECT last_attempt_ip FROM account WHERE id = ?;

-- :name sel_last_ip
SELECT last_ip FROM account WHERE id = ?;

-- :name sel_realmlist_security_level
SELECT allowedSecurityLevel from realmlist WHERE id = ?;

-- :name del_account
DELETE FROM account WHERE id = ?;

-- :name sel_autobroadcast
SELECT id, weight, text FROM autobroadcast WHERE realmid = ? OR realmid = -1;

-- :name get_email_by_id :typed :?
SELECT email FROM account WHERE id = ?;

-- :name ins_aldl_ip_logging
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: uint32, 4: string
-- Complete name: Login_Insert_AccountLoginDeLete_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, (SELECT last_ip FROM account WHERE id = ?), ?, unix_timestamp(NOW()), NOW());


-- :name ins_facl_ip_logging
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: uint32, 4: string
-- Complete name: Login_Insert_FailedAccountLogin_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, (SELECT last_attempt_ip FROM account WHERE id = ?), ?, unix_timestamp(NOW()), NOW());

-- :name ins_char_ip_logging
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: string, 4: string
-- Complete name: Login_Insert_CharacterDelete_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, ?, ?, unix_timestamp(NOW()), NOW());

-- :name sel_rbac_account_permissions :typed :*
SELECT permissionId, granted FROM rbac_account_permissions WHERE accountId = ? AND (realmId = ? OR realmId = -1) ORDER BY permissionId, realmId;

-- :name ins_rbac_account_permission
INSERT INTO rbac_account_permissions (accountId, permissionId, granted, realmId) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE granted = VALUES(granted);

-- :name del_rbac_account_permission
DELETE FROM rbac_account_permissions WHERE accountId = ? AND permissionId = ? AND (realmId = ? OR realmId = -1);

-- :name ins_account_mute
INSERT INTO account_muted VALUES (?, UNIX_TIMESTAMP(), ?, ?, ?);

-- :name sel_account_mute_info :typed :*
SELECT mutedate, mutetime, mutereason, mutedby FROM account_muted WHERE guid = ? ORDER BY mutedate ASC;

-- :name del_account_muted
DELETE FROM account_muted WHERE guid = ?;

-- :name sel_bnet_authentication :typed :?
SELECT ba.id, ba.sha_pass_hash, ba.failed_logins, ba.LoginTicket, ba.LoginTicketExpiry, (bab.unbandate > UNIX_TIMESTAMP() OR bab.unbandate = bab.bandate) as is_banned FROM battlenet_accounts ba LEFT JOIN battlenet_account_bans bab ON ba.id = bab.id WHERE email = ?;

-- :name upd_bnet_authentication,
UPDATE battlenet_accounts SET LoginTicket = ?, LoginTicketExpiry = ? WHERE id = ?;

-- :name sel_bnet_existing_authentication :?
SELECT LoginTicketExpiry FROM battlenet_accounts WHERE LoginTicket = ?;

-- :name upd_bnet_existing_authentication
UPDATE battlenet_accounts SET LoginTicketExpiry = ? WHERE LoginTicket = ?;

-- :name sel_bnet_account_info :*
-- :doc 0: bytes like
SELECT
-- BnetAccountInfo
ba.id, UPPER(ba.email), ba.locked, ba.lock_country, ba.last_ip, ba.LoginTicketExpiry, bab.unbandate > UNIX_TIMESTAMP() OR bab.unbandate = bab.bandate, bab.unbandate = bab.bandate,
-- BnetGameAccountInfo
a.id, a.username, ab.unbandate, ab.unbandate = ab.bandate, aa.gmlevel
FROM battlenet_accounts ba
LEFT JOIN battlenet_account_bans bab ON ba.id = bab.id
LEFT JOIN account a ON ba.id = a.battlenet_account
LEFT JOIN account_banned ab ON a.id = ab.id AND ab.active = 1
LEFT JOIN account_access aa ON a.id = aa.id AND aa.RealmID = -1
WHERE
    ba.LoginTicket = ?
ORDER BY a.id;

-- :name upd_bnet_last_login_info
UPDATE battlenet_accounts SET last_ip = ?, last_login = NOW(), locale = ?, failed_logins = 0, os = ? WHERE id = ?;

-- :name upd_bnet_game_account_login_info
UPDATE account SET sessionkey = ?, last_ip = ?, last_login = NOW(), locale = ?, failed_logins = 0, os = ? WHERE username = ?;

-- :name sel_bnet_character_counts_by_account_id
SELECT rc.acctid, rc.numchars, r.id, r.Region, r.Battlegroup FROM realmcharacters rc INNER JOIN realmlist r ON rc.realmid = r.id WHERE rc.acctid = ?;

-- :name sel_bnet_character_counts_by_bnet_id :*
-- :doc 0: battlenet account ID
SELECT rc.acctid, rc.numchars, r.id, r.Region, r.Battlegroup FROM realmcharacters rc INNER JOIN realmlist r ON rc.realmid = r.id LEFT JOIN account a ON rc.acctid = a.id WHERE a.battlenet_account = ?;

-- :name sel_bnet_last_player_characters :*
-- :doc 0: battlenet account ID
SELECT lpc.accountId, lpc.region, lpc.battlegroup, lpc.realmId, lpc.characterName, lpc.characterGUID, lpc.lastPlayedTime FROM account_last_played_character lpc LEFT JOIN account a ON lpc.accountId = a.id WHERE a.battlenet_account = ?;

-- :name del_bnet_last_player_characters
DELETE FROM account_last_played_character WHERE accountId = ? AND region = ? AND battlegroup = ?;

-- :name ins_bnet_last_player_characters
INSERT INTO account_last_played_character (accountId, region, battlegroup, realmId, characterName, characterGUID, lastPlayedTime) VALUES (?,?,?,?,?,?,?);

-- :name ins_bnet_account
INSERT INTO battlenet_accounts (`email`,`sha_pass_hash`) VALUES (?, ?);

-- :name sel_bnet_account_email_by_id :typed :?
SELECT email FROM battlenet_accounts WHERE id = ?;

-- :name sel_bnet_account_id_by_email :typed :?
SELECT id FROM battlenet_accounts WHERE email = ?;

-- :name upd_bnet_password
UPDATE battlenet_accounts SET sha_pass_hash = ? WHERE id = ?;

-- :name sel_bnet_check_password :?
SELECT 1 FROM battlenet_accounts WHERE id = ? AND sha_pass_hash = ?;

-- :name upd_bnet_account_lock
UPDATE battlenet_accounts SET locked = ? WHERE id = ?;

-- :name upd_bnet_account_lock_country
UPDATE battlenet_accounts SET lock_country = ? WHERE id = ?;

-- :name sel_bnet_account_id_by_game_account :typed :?
SELECT battlenet_account FROM account WHERE id = ?;

-- :name upd_bnet_game_account_link
UPDATE account SET battlenet_account = ?, battlenet_index = ? WHERE id = ?;

-- :name sel_bnet_max_account_index :typed :?
SELECT MAX(battlenet_index) as bnet_max_index FROM account WHERE battlenet_account = ?;

-- :name sel_bnet_game_account_list_small
SELECT a.id, a.username FROM account a LEFT JOIN battlenet_accounts ba ON a.battlenet_account = ba.id WHERE ba.email = ?;

-- :name sel_bnet_game_account_list :*
SELECT a.username, a.expansion, ab.bandate, ab.unbandate, ab.banreason FROM account AS a LEFT JOIN account_banned AS ab ON a.id = ab.id AND ab.active = 1 INNER JOIN battlenet_accounts AS ba ON a.battlenet_account = ba.id WHERE ba.LoginTicket = ? ORDER BY a.id;

-- :name upd_bnet_failed_logins
UPDATE battlenet_accounts SET failed_logins = failed_logins + 1 WHERE id = ?;

-- :name ins_bnet_account_auto_banned
INSERT INTO battlenet_account_bans(id, bandate, unbandate, bannedby, banreason) VALUES(?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, 'Azothacore Auth', 'Failed login autoban');

-- :name del_bnet_expired_account_banned
DELETE FROM battlenet_account_bans WHERE unbandate<>bandate AND unbandate<=UNIX_TIMESTAMP();

-- :name upd_bnet_reset_failed_logins
UPDATE battlenet_accounts SET failed_logins = 0 WHERE id = ?;


-- :name sel_last_char_undelete
SELECT LastCharacterUndelete FROM battlenet_accounts WHERE Id = ?;

-- :name upd_last_char_undelete
UPDATE battlenet_accounts SET LastCharacterUndelete = UNIX_TIMESTAMP() WHERE Id = ?;

-- Account wide toys

-- :name sel_account_toys
SELECT itemId, isFavourite FROM battlenet_account_toys WHERE accountId = ?;

-- :name rep_account_toys
REPLACE INTO battlenet_account_toys (accountId, itemId, isFavourite) VALUES (?, ?, ?);


-- Battle Pets

-- :name sel_battle_pets
SELECT guid, species, breed, level, exp, health, quality, flags, name FROM battle_pets WHERE battlenetAccountId = ?;

-- :name ins_battle_pets
INSERT INTO battle_pets (guid, battlenetAccountId, species, breed, level, exp, health, quality, flags, name) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- :name del_battle_pets
DELETE FROM battle_pets WHERE battlenetAccountId = ? AND guid = ?;

-- :name upd_battle_pets
UPDATE battle_pets SET level = ?, exp = ?, health = ?, quality = ?, flags = ?, name = ? WHERE battlenetAccountId = ? AND guid = ?;

-- :name sel_battle_pet_slots
SELECT id, battlePetGuid, locked FROM battle_pet_slots WHERE battlenetAccountId = ?;

-- :name ins_battle_pet_slots
INSERT INTO battle_pet_slots (id, battlenetAccountId, battlePetGuid, locked) VALUES (?, ?, ?, ?);

-- :name del_battle_pet_slots
DELETE FROM battle_pet_slots WHERE battlenetAccountId = ?;


-- :name sel_account_heirlooms
SELECT itemId, flags FROM battlenet_account_heirlooms WHERE accountId = ?;

-- :name rep_account_heirlooms
REPLACE INTO battlenet_account_heirlooms (accountId, itemId, flags) VALUES (?, ?, ?);


-- Account wide mounts

-- :name sel_account_mounts
SELECT mountSpellId, flags FROM battlenet_account_mounts WHERE battlenetAccountId = ?;

-- :name rep_account_mounts
REPLACE INTO battlenet_account_mounts (battlenetAccountId, mountSpellId, flags) VALUES (?, ?, ?);

-- Transmog collection

-- :name sel_bnet_item_appearances
SELECT blobIndex, appearanceMask FROM battlenet_item_appearances WHERE battlenetAccountId = ? ORDER BY blobIndex DESC;

-- :name ins_bnet_item_appearances
INSERT INTO battlenet_item_appearances (battlenetAccountId, blobIndex, appearanceMask) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE appearanceMask = appearanceMask | VALUES(appearanceMask);

-- :name sel_bnet_item_favorite_appearances
SELECT itemModifiedAppearanceId FROM battlenet_item_favorite_appearances WHERE battlenetAccountId = ?;

-- :name ins_bnet_item_favorite_appearance
INSERT INTO battlenet_item_favorite_appearances (battlenetAccountId, itemModifiedAppearanceId) VALUES (?, ?);

-- :name del_bnet_item_favorite_appearance
DELETE FROM battlenet_item_favorite_appearances WHERE battlenetAccountId = ? AND itemModifiedAppearanceId = ?;

-- NOTE: IGNORED QUERIES FROM TC BECAUSE WE DONT WANT TO IMPL stuff, such as IP2NATION, or DB Logging or TOTP etc first
--          0) LOGIN_SEL_LOGON_COUNTRY, "SELECT country FROM ip2nation WHERE ip < ? ORDER BY ip DESC LIMIT 0,1", CONNECTION_SYNCH);
--          1) LOGIN_INS_LOG INSERT INTO logs (time, realm, type, level, string) VALUES (?, ?, ?, ?, ?);
--          2) LOGIN_DEL_OLD_LOGS, "DELETE FROM logs WHERE (time + ?) < ? AND realm = ?", CONNECTION_ASYNC);
--          3) LOGIN_SEL_IP2NATION_COUNTRY, "SELECT c.country FROM ip2nationCountries c, ip2nation i WHERE i.ip < ? AND c.code = i.country ORDER BY i.ip DESC LIMIT 0,1", CONNECTION_SYNCH);
--          4) 
