
-- :name sel_logonchallenge
SELECT a.id, a.username, a.locked, a.lock_country, a.last_ip, a.failed_logins, 
ab.unbandate > UNIX_TIMESTAMP() OR ab.unbandate = ab.bandate, ab.unbandate = ab.bandate, 
ipb.unbandate > UNIX_TIMESTAMP() OR ipb.unbandate = ipb.bandate, ipb.unbandate = ipb.bandate, 
aa.gmlevel, a.totp_secret, a.salt, a.verifier 
FROM account a 
LEFT JOIN account_access aa ON a.id = aa.id 
LEFT JOIN account_banned ab ON ab.id = a.id AND ab.active = 1 
LEFT JOIN ip_banned ipb ON ipb.ip = ? 
WHERE a.username = ?;

-- :name sel_reconnectchallenge
SELECT a.id, a.username, a.locked, a.lock_country, a.last_ip, a.failed_logins, 
ab.unbandate > UNIX_TIMESTAMP() OR ab.unbandate = ab.bandate, ab.unbandate = ab.bandate, 
ipb.unbandate > UNIX_TIMESTAMP() OR ipb.unbandate = ipb.bandate, ipb.unbandate = ipb.bandate, 
aa.gmlevel, a.session_key 
FROM account a 
LEFT JOIN account_access aa ON a.id = aa.id 
LEFT JOIN account_banned ab ON ab.id = a.id AND ab.active = 1 
LEFT JOIN ip_banned ipb ON ipb.ip = ? 
WHERE a.username = ? AND a.session_key IS NOT NULL;

-- :name sel_account_info_by_name
SELECT a.id, a.session_key, a.last_ip, a.locked, a.lock_country, a.expansion, a.mutetime, a.locale, a.recruiter, a.os, a.totaltime, 
aa.gmlevel, ab.unbandate > UNIX_TIMESTAMP() OR ab.unbandate = ab.bandate, r.id FROM account a LEFT JOIN account_access aa ON a.id = aa.id AND aa.RealmID IN (-1, ?) 
LEFT JOIN account_banned ab ON a.id = ab.id AND ab.active = 1 LEFT JOIN account r ON a.id = r.recruiter WHERE a.username = ? 
AND a.session_key IS NOT NULL ORDER BY aa.RealmID DESC LIMIT 1;

-- :name sel_ip_info
SELECT unbandate > UNIX_TIMESTAMP() OR unbandate = bandate AS banned, NULL as country FROM ip_banned WHERE ip = ?;

-- :name sel_realmlist
SELECT id, name, address, localAddress, localSubnetMask, port, icon, flag, timezone, allowedSecurityLevel, population, gamebuild FROM realmlist WHERE flag <> 3 ORDER BY name;

-- :name del_expired_ip_bans
DELETE FROM ip_banned WHERE unbandate<>bandate AND unbandate<=UNIX_TIMESTAMP();

-- :name upd_expired_account_bans
UPDATE account_banned SET active = 0 WHERE active = 1 AND unbandate<>bandate AND unbandate<=UNIX_TIMESTAMP();

-- :name sel_ip_banned
SELECT * FROM ip_banned WHERE ip = ?;

-- :name ins_ip_auto_banned
INSERT INTO ip_banned (ip, bandate, unbandate, bannedby, banreason) VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, 'Trinity realmd', 'Failed login autoban');

-- :name sel_ip_banned_all
SELECT ip, bandate, unbandate, bannedby, banreason FROM ip_banned WHERE (bandate = unbandate OR unbandate > UNIX_TIMESTAMP()) ORDER BY unbandate;

-- :name sel_ip_banned_by_ip
SELECT ip, bandate, unbandate, bannedby, banreason FROM ip_banned WHERE (bandate = unbandate OR unbandate > UNIX_TIMESTAMP()) AND ip LIKE CONCAT('%%', ?, '%%') ORDER BY unbandate;

-- :name sel_account_banned
SELECT bandate, unbandate FROM account_banned WHERE id = ? AND active = 1;

-- :name sel_account_banned_all
SELECT account.id, username FROM account, account_banned WHERE account.id = account_banned.id AND active = 1 GROUP BY account.id;

-- :name sel_account_banned_by_username
SELECT account.id, username FROM account, account_banned WHERE account.id = account_banned.id AND active = 1 AND username LIKE CONCAT('%%', ?, '%%') GROUP BY account.id;

-- :name ins_account_auto_banned
INSERT INTO account_banned VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, 'Trinity realmd', 'Failed login autoban', 1);

-- :name del_account_banned
DELETE FROM account_banned WHERE id = ?;

-- :name upd_logon
UPDATE account SET salt = ?, verifier = ? WHERE id = ?;

-- :name upd_logonproof
UPDATE account SET session_key = ?, last_ip = ?, last_login = NOW(), locale = ?, failed_logins = 0, os = ? WHERE username = ?;

-- :name upd_failedlogins
UPDATE account SET failed_logins = failed_logins + 1 WHERE username = ?;

-- :name sel_failedlogins
SELECT id, failed_logins FROM account WHERE username = ?;

-- :name sel_account_id_by_name
SELECT id FROM account WHERE username = ?;

-- :name sel_account_list_by_name
SELECT id, username FROM account WHERE username = ?;

-- :name sel_account_list_by_email
SELECT id, username FROM account WHERE email = ?;

-- :name sel_num_chars_on_realm
SELECT numchars FROM realmcharacters WHERE realmid = ? AND acctid= ?;

-- :name sel_realm_character_counts
SELECT realmid, numchars FROM realmcharacters WHERE acctid = ?;

-- :name sel_account_by_ip
SELECT id, username FROM account WHERE last_ip = ?;

-- :name sel_account_by_id
SELECT 1 FROM account WHERE id = ?;

-- :name ins_ip_banned
INSERT INTO ip_banned (ip, bandate, unbandate, bannedby, banreason) VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, ?, ?);

-- :name del_ip_not_banned
DELETE FROM ip_banned WHERE ip = ?;

-- :name ins_account_banned
INSERT INTO account_banned VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP()+?, ?, ?, 1);

-- :name upd_account_not_banned
UPDATE account_banned SET active = 0 WHERE id = ? AND active != 0;

-- :name del_realm_characters
DELETE FROM realmcharacters WHERE acctid = ?;

-- :name rep_realm_characters
REPLACE INTO realmcharacters (numchars, acctid, realmid) VALUES (?, ?, ?);

-- :name sel_sum_realm_characters
SELECT SUM(numchars) FROM realmcharacters WHERE acctid = ?;

-- :name ins_account
INSERT INTO account(username, salt, verifier, expansion, joindate) VALUES(?, ?, ?, ?, NOW());

-- :name ins_realm_characters_init
INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid=account.id WHERE acctid IS NULL;

-- :name upd_expansion
UPDATE account SET expansion = ? WHERE id = ?;

-- :name upd_account_lock
UPDATE account SET locked = ? WHERE id = ?;

-- :name upd_account_lock_country
UPDATE account SET lock_country = ? WHERE id = ?;

-- :name upd_username
UPDATE account SET username = ? WHERE id = ?;

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

-- :name del_old_logs
DELETE FROM logs WHERE (time + ?) < ?;

-- :name del_account_access
DELETE FROM account_access WHERE id = ?;

-- :name del_account_access_by_realm
DELETE FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1);

-- :name ins_account_access
INSERT INTO account_access (id,gmlevel,RealmID) VALUES (?, ?, ?);

-- :name get_account_id_by_username
SELECT id FROM account WHERE username = ?;

-- :name get_account_access_gmlevel
SELECT gmlevel FROM account_access WHERE id = ?;

-- :name get_gmlevel_by_realmid
SELECT gmlevel FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1);

-- :name get_username_by_id
SELECT username FROM account WHERE id = ?;

-- :name sel_check_password
SELECT salt, verifier FROM account WHERE id = ?;

-- :name sel_check_password_by_name
SELECT salt, verifier FROM account WHERE username = ?;

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

-- :name sel_account_recruiter
SELECT 1 FROM account WHERE recruiter = ?;

-- :name sel_bans
SELECT 1 FROM account_banned WHERE id = ? AND active = 1 UNION SELECT 1 FROM ip_banned WHERE ip = ?;

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

-- :name sel_motd
SELECT text FROM motd WHERE realmid = ? OR realmid = -1 ORDER BY realmid DESC;

-- :name rep_motd
REPLACE INTO motd (realmid, text) VALUES (?, ?);

-- :name ins_account_mute
INSERT INTO account_muted VALUES (?, UNIX_TIMESTAMP(), ?, ?, ?);

-- :name sel_account_mute_info
SELECT mutedate, mutetime, mutereason, mutedby FROM account_muted WHERE guid = ? ORDER BY mutedate ASC;

-- :name del_account_muted
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: uint32, 4: string
-- Complete name: Login_Insert_AccountLoginDeLete_IP_Logging
DELETE FROM account_muted WHERE guid = ?;

-- :name ins_aldl_ip_logging
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: uint32, 4: string
-- Complete name: Login_Insert_FailedAccountLogin_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, (SELECT last_ip FROM account WHERE id = ?), ?, unix_timestamp(NOW()), NOW());

-- :name ins_facl_ip_logging
-- :doc 0: uint32, 1: uint32, 2: uint8, 3: string, 4: string
-- Complete name: Login_Insert_CharacterDelete_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, (SELECT last_attempt_ip FROM account WHERE id = ?), ?, unix_timestamp(NOW()), NOW());

-- :name ins_char_ip_logging
-- :doc 0: string, 1: string, 2: string
-- Complete name: Login_Insert_Failed_Account_Login_due_password_IP_Logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, ?, ?, ?, ?, unix_timestamp(NOW()), NOW());

-- :name ins_falp_ip_logging
INSERT INTO logs_ip_actions (account_id,character_guid,type,ip,systemnote,unixtime,time) VALUES (?, 0, 1, ?, ?, unix_timestamp(NOW()), NOW());

-- >>>>> DB logging

-- :name ins_log
INSERT INTO logs (time, realm, type, level, string) VALUES (?, ?, ?, ?, ?);

-- >>>>> TOTP

-- :name sel_secret_digest
SELECT digest FROM secret_digest WHERE id = ?;

-- :name ins_secret_digest
INSERT INTO secret_digest (id, digest) VALUES (?,?);

-- :name del_secret_digest
DELETE FROM secret_digest WHERE id = ?;


-- :name sel_account_totp_secret
SELECT totp_secret FROM account WHERE id = ?;

-- :name upd_account_totp_secret
UPDATE account SET totp_secret = ? WHERE id = ?;