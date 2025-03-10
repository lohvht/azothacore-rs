/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `transports`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `transports` (
  `guid` bigint(20) unsigned NOT NULL DEFAULT 0,
  `entry` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `name` text DEFAULT NULL,
  `phaseUseFlags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `phaseid` int(10) NOT NULL DEFAULT 0,
  `phasegroup` int(10) NOT NULL DEFAULT 0,
  `ScriptName` char(64) NOT NULL DEFAULT '',
  PRIMARY KEY (`guid`),
  UNIQUE KEY `idx_entry` (`entry`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci COMMENT='Transports';
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `transports` WRITE;
/*!40000 ALTER TABLE `transports` DISABLE KEYS */;
INSERT INTO `transports` VALUES
(1,176495,'Undercity, Tirisfal Glades and Grom\'gol Base Camp, Stranglethorn Vale (\"The Purple Princess\")',0,0,0,''),
(2,176310,'Stormwind Harbor and Auberdine, Darkshore (\"Ship (The Bravery)\")',0,0,0,''),
(4,176231,'Menethil Harbor, Wetlands and Theramore Isle, Dustwallow Marsh (\"The Lady Mehley\")',0,0,0,''),
(5,175080,'Orgrimmar, Durotar and Grom\'gol Base Camp, Stranglethorn Vale (\"The Iron Eagle\")',0,0,0,''),
(6,164871,'Orgrimmar, Durotar and Undercity, Tirisfal Glades (\"The Thundercaller\")',0,0,0,''),
(7,20808,'Steamwheedle Cartel ports, Ratchet and Booty Bay (\"The Maiden\'s Fancy\")',0,0,0,''),
(8,177233,'The Forgotten Coast, Feralas and Feathermoon Stronghold, Sardor Isle, Feralas (\"Feathermoon Ferry\")',0,0,0,''),
(9,181646,'Valaar\'s Berth, Azuremyst Isle and Auberdine, Darkshore (\"Elune\'s Blessing\")',0,0,0,''),
(10,181688,'Menethil Harbor, Wetlands and Valgarde, Howling Fjord (\"Northspear\")',0,0,0,''),
(11,181689,'Undercity, Tirisfal Glades and Vengeance Landing, Howling Fjord (\"Zeppelin, Horde (Cloudkisser)\")',0,0,0,''),
(12,186238,'Orgrimmar, Durotar and Warsong Hold, Borean Tundra (\"Zeppelin, Horde (The Mighty Wind)\")',0,0,0,''),
(13,186371,'Westguard Keep in Howling Fjord to bombard pirate (\"Zeppelin\")',0,0,0,''),
(14,187038,'Not Boardable - Cyrcling in Howling Fjord (\"Sister Mercy\")',0,0,0,''),
(15,187568,'Unu\'pe, Borean Tundra and Moa\'ki Harbor, Dragonblight (\"Turtle (Walker of Waves)\")',0,0,0,''),
(16,188511,'Moa\'ki Harbor and Kamagua (\"Turtle (Green Island)\")',0,0,0,''),
(17,190536,'Stormwing Harbor and Valiance Keep, Borean Tundra (\"The Kraken\")',0,0,0,''),
(18,192241,'Horde gunship patrolling above Icecrown (\"Orgrim\'s Hammer\")',0,0,0,''),
(19,192242,'Alliance gunship patrolling above Icecrown (\"The Skybreaker\")',0,0,0,''),
(20,190549,'Orgrimmar and Thunder Bluff',0,0,0,''),
(21,206328,'Krazzworks to Dragonmaw Port',0,0,0,''),
(22,206329,'Dragonmaw Port to Krazzworks',0,0,0,''),
(23,203466,'Ship to Vashj\'ir - (Horde)',0,0,0,''),
(24,203626,'The Spear of Durotar',0,0,0,''),
(25,197195,'Ship to Vashj\'ir - (Alliance)',0,0,0,''),
(26,207227,'Krazzworks Attack Zeppelin',0,0,0,''),
(27,204018,'Deepholm - Alliance Gunship',0,0,0,''),
(28,203428,'Worgen area - Orc Gunship',0,0,0,'');
/*!40000 ALTER TABLE `transports` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

