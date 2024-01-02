/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `game_event`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `game_event` (
  `eventEntry` tinyint(3) unsigned NOT NULL COMMENT 'Entry of the game event',
  `start_time` timestamp NULL DEFAULT NULL COMMENT 'Absolute start date, the event will never start before',
  `end_time` timestamp NULL DEFAULT NULL COMMENT 'Absolute end date, the event will never start after',
  `occurence` bigint(20) unsigned NOT NULL DEFAULT 5184000 COMMENT 'Delay in minutes between occurences of the event',
  `length` bigint(20) unsigned NOT NULL DEFAULT 2592000 COMMENT 'Length in minutes of the event',
  `holiday` mediumint(8) unsigned NOT NULL DEFAULT 0 COMMENT 'Client side holiday id',
  `description` varchar(255) DEFAULT NULL COMMENT 'Description of the event displayed in console',
  `world_event` tinyint(3) unsigned NOT NULL DEFAULT 0 COMMENT '0 if normal event, 1 if world event',
  `announce` tinyint(3) unsigned DEFAULT 2 COMMENT '0 dont announce, 1 announce, 2 value from config',
  PRIMARY KEY (`eventEntry`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `game_event` WRITE;
/*!40000 ALTER TABLE `game_event` DISABLE KEYS */;
INSERT INTO `game_event` VALUES
(1,'2017-06-20 23:01:00','2020-12-31 05:00:00',525600,20160,341,'Midsummer Fire Festival',0,2),
(2,'2016-12-15 06:00:00','2020-12-31 05:00:00',525600,25920,141,'Winter Veil',0,2),
(3,'2016-11-06 00:01:00','2020-12-31 04:00:00',43200,10078,479,'Darkmoon Faire',0,2),
(6,'2016-12-31 06:00:00','2020-12-31 05:00:00',525600,1440,0,'New Year\'s Eve',0,2),
(7,'2017-01-21 00:01:00','2020-12-31 05:00:00',525600,20160,327,'Lunar Festival',0,2),
(8,'2017-02-07 00:01:00','2020-12-31 05:00:00',525600,20160,423,'Love is in the Air',0,2),
(9,'2017-03-17 00:01:00','2020-12-31 05:00:00',524160,10080,181,'Noblegarden',0,2),
(10,'2017-04-30 23:01:00','2020-12-31 05:00:00',525600,10080,201,'Children\'s Week ',0,2),
(11,'2017-09-28 23:01:00','2020-12-31 05:00:00',525600,10080,321,'Harvest Festival',0,2),
(12,'2016-10-18 00:00:00','2020-12-31 05:00:00',525600,20160,324,'Hallow\'s End',0,2),
(13,NULL,NULL,525600,1,0,'Elemental Invasions',0,2),
(14,'2016-10-28 23:00:00','2020-12-31 05:00:00',10080,1440,0,'Stranglethorn Fishing Extravaganza Announce',0,2),
(15,'2016-10-30 14:00:00','2020-12-31 05:00:00',10080,120,301,'Stranglethorn Fishing Extravaganza Fishing Pools',0,2),
(16,'2016-10-29 02:00:00','2020-12-31 05:00:00',180,120,0,'Gurubashi Arena Booty Run',0,2),
(17,NULL,NULL,525600,1,0,'Scourge Invasion',0,2),
(18,'2016-11-25 00:01:00','2020-12-31 05:00:00',60480,5760,283,'Call to Arms: Alterac Valley!',0,2),
(19,'2016-12-02 00:01:00','2020-12-31 05:00:00',60480,5760,284,'Call to Arms: Warsong Gulch!',0,2),
(20,'2016-11-11 00:01:00','2020-12-31 05:00:00',60480,5760,285,'Call to Arms: Arathi Basin!',0,2),
(21,'2016-11-18 00:01:00','2020-12-31 05:00:00',60480,5760,353,'Call to Arms: Eye of the Storm!',0,2),
(22,NULL,NULL,525600,1,0,'AQ War Effort',0,2),
(23,'2016-12-01 00:01:00','2020-12-31 05:00:00',43200,4320,0,'Darkmoon Faire Building',0,2),
(24,'2017-09-19 23:01:00','2020-12-31 05:00:00',525600,21600,372,'Brewfest',0,2),
(25,'2016-10-28 20:00:00','2020-12-31 05:00:00',1440,720,0,'Nights',0,2),
(26,'2017-11-21 00:01:00','2020-12-31 05:00:00',525600,10020,404,'Pilgrim\'s Bounty',0,2),
(27,'2008-03-24 05:00:00','2020-12-31 05:00:00',86400,21600,0,'Edge of Madness, Gri\'lek',0,2),
(28,'2008-04-07 05:00:00','2020-12-31 05:00:00',86400,21600,0,'Edge of Madness, Hazza\'rah',0,2),
(29,'2008-04-21 05:00:00','2020-12-31 05:00:00',86400,21600,0,'Edge of Madness, Renataki',0,2),
(30,'2008-05-05 05:00:00','2020-12-31 05:00:00',86400,21600,0,'Edge of Madness, Wushoolay',0,2),
(31,NULL,NULL,5184000,2592000,0,'Arena Tournament',0,2),
(32,'2014-09-18 22:00:01','2020-01-01 07:00:00',10080,5,0,'L70ETC Concert',0,2),
(33,'2011-03-21 23:10:00','2020-03-21 23:00:00',30,5,0,'Dalaran: Minigob',0,2),
(34,'2016-09-30 23:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month October',0,2),
(35,'2016-11-01 00:01:00','2020-12-31 05:00:00',525600,43200,0,'Brew of the Month November',0,2),
(36,'2016-12-01 00:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month December',0,2),
(37,'2017-01-01 00:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month January',0,2),
(38,'2017-02-01 00:01:00','2020-12-31 05:00:00',525600,40320,0,'Brew of the Month February',0,2),
(39,'2017-03-01 00:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month March',0,2),
(40,'2017-03-31 23:01:00','2020-12-31 05:00:00',525600,43200,0,'Brew of the Month April',0,2),
(41,'2017-04-30 23:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month May',0,2),
(42,'2017-05-31 23:01:00','2020-12-31 05:00:00',525600,43200,0,'Brew of the Month June',0,2),
(43,'2017-06-30 23:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month July',0,2),
(44,'2017-07-31 23:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month August',0,2),
(45,'2017-08-31 23:01:00','2020-12-31 05:00:00',525600,44640,0,'Brew of the Month September',0,2),
(48,NULL,NULL,5184000,2592000,0,'Wintergrasp Alliance Defence',5,2),
(49,NULL,NULL,5184000,2592000,0,'Wintergrasp Horde Defence',5,2),
(50,'2017-09-18 23:01:00','2020-12-31 04:00:00',525600,1440,398,'Pirates\' Day',0,2),
(51,'2016-11-01 00:01:00','2020-12-31 05:00:00',525600,2820,409,'Day of the Dead',0,2),
(52,'2010-12-25 05:00:00','2020-12-31 05:00:00',525600,11700,0,'Winter Veil: Gifts',0,2),
(53,'2016-10-27 23:01:00','2020-12-31 09:00:00',60480,6240,400,'Call to Arms: Strand of the Ancients!',0,2),
(54,'2016-11-04 00:01:00','2020-12-31 09:00:00',60480,6240,420,'Call to Arms: Isle of Conquest!',0,2),
(55,NULL,NULL,5184000,2592000,0,'Arena Season 3',0,2),
(56,NULL,NULL,5184000,2592000,0,'Arena Season 4',0,2),
(57,NULL,NULL,5184000,2592000,0,'Arena Season 5',0,2),
(58,NULL,NULL,5184000,2592000,0,'Arena Season 6',0,2),
(59,NULL,NULL,5184000,2592000,0,'Arena Season 7',0,2),
(60,NULL,NULL,5184000,2592000,0,'Arena Season 8',0,2),
(61,'2010-09-06 23:00:00','2010-10-09 23:00:00',9999999,47520,0,'Zalazane\'s Fall',0,2),
(62,'2016-10-30 14:00:00','2020-12-31 05:00:00',10080,180,0,'Stranglethorn Fishing Extravaganza Turn-ins',0,2),
(63,'2016-10-29 12:00:00','2020-12-31 05:00:00',10080,180,424,'Kalu\'ak Fishing Derby Turn-ins',0,2),
(64,'2016-10-29 13:00:00','2020-12-31 05:00:00',10080,60,0,'Kalu\'ak Fishing Derby Fishing Pools',0,2),
(65,'2008-01-02 14:55:00','2020-12-31 05:00:00',240,15,0,'Perry Gatner',0,2),
(67,'2010-01-01 23:40:00','2020-12-31 05:00:00',60,5,0,'AT Event Trigger (Tirion Speech)',0,2),
(68,'2010-01-01 23:55:00','2020-12-31 05:00:00',60,5,0,'AT Event Trigger (Horde Event)',0,2),
(69,'2010-01-01 23:10:00','2020-12-31 05:00:00',60,5,0,'AT Event Trigger (Alliance Event)',0,2),
(70,'2013-11-17 23:01:00','2013-12-01 22:59:00',525600,10080,0,'WoW\'s 9th Anniversary',0,2),
(71,'2014-11-16 23:01:00','2014-12-01 22:59:00',525600,10080,0,'WoW\'s 10th Anniversary',0,2),
(72,'2010-07-04 10:00:00','2020-07-04 10:00:00',525600,1440,62,'Fireworks Spectacular',0,2);
/*!40000 ALTER TABLE `game_event` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

