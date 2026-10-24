/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `pool_pool`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `pool_pool` (
  `pool_id` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `mother_pool` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `chance` float NOT NULL DEFAULT 0,
  `description` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`pool_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `pool_pool` WRITE;
/*!40000 ALTER TABLE `pool_pool` DISABLE KEYS */;
INSERT INTO `pool_pool` VALUES
(4872,2017,0,'Spawn Point 3 - Isle of Quel\'Danas'),
(4873,2017,0,'Spawn Point 4 - Isle of Quel\'Danas'),
(5123,5122,0,'Cobalt Dragonblight'),
(10020,9900,0,'Borean Tundra Ore Spawnpoint 21'),
(10161,9901,0,'Howling Fjord Ore Spawnpoint 12'),
(10162,9901,0,'Howling Fjord Ore Spawnpoint 13'),
(10651,9904,0,'Zul\'Drak Ore Spawnpoint 2'),
(10655,9904,0,'Zul\'Drak Ore Spawnpoint 6'),
(10687,9904,0,'Zul\'Drak Ore Spawnpoint 38'),
(10702,9904,0,'Zul\'Drak Ore Spawnpoint 53'),
(10740,9904,0,'Zul\'Drak Ore Spawnpoint 91'),
(11405,9908,0,'Icecrown Ore Spawnpoint 6'),
(11406,9908,0,'Icecrown Ore Spawnpoint 7'),
(11412,9908,0,'Icecrown Ore Spawnpoint 13'),
(11413,9908,0,'Icecrown Ore Spawnpoint 14'),
(11614,9908,0,'Icecrown Ore Spawnpoint 215'),
(11616,9908,0,'Icecrown Ore Spawnpoint 217'),
(11617,9908,0,'Icecrown Ore Spawnpoint 218'),
(11622,9908,0,'Icecrown Ore Spawnpoint 223'),
(32491,60002,20,'Time-Lost Proto Drake (32491)'),
(32630,60002,80,'Vyragosa (32630)'),
(51106,5101,0,'Searing Gorge Ore Spawnpoint'),
(51117,5101,0,'Searing Gorge Ore Spawnpoint'),
(51126,5101,0,'Searing Gorge Ore Spawnpoint'),
(51129,5101,0,'Searing Gorge Ore Spawnpoint'),
(51139,5101,0,'Searing Gorge Ore Spawnpoint'),
(51140,5101,0,'Searing Gorge Ore Spawnpoint'),
(51150,5101,0,'Searing Gorge Ore Spawnpoint'),
(51163,5101,0,'Searing Gorge Ore Spawnpoint'),
(51168,5101,0,'Searing Gorge Ore Spawnpoint'),
(51176,5101,0,'Searing Gorge Ore Spawnpoint'),
(51199,5101,0,'Searing Gorge Ore Spawnpoint'),
(51233,5101,0,'Searing Gorge Ore Spawnpoint'),
(51238,5101,0,'Searing Gorge Ore Spawnpoint'),
(51257,5101,0,'Searing Gorge Ore Spawnpoint'),
(51269,5101,0,'Searing Gorge Ore Spawnpoint'),
(51270,5101,0,'Searing Gorge Ore Spawnpoint'),
(51275,5101,0,'Searing Gorge Ore Spawnpoint'),
(51284,5101,0,'Searing Gorge Ore Spawnpoint'),
(361010,36101,0,'Felwood Ore Spawnpoint'),
(361012,36101,0,'Felwood Ore Spawnpoint'),
(361025,36101,0,'Felwood Ore Spawnpoint'),
(361053,36101,0,'Felwood Ore Spawnpoint'),
(361054,36101,0,'Felwood Ore Spawnpoint'),
(361057,36101,0,'Felwood Ore Spawnpoint'),
(361059,36101,0,'Felwood Ore Spawnpoint'),
(361075,36101,0,'Felwood Ore Spawnpoint'),
(361079,36101,0,'Felwood Ore Spawnpoint'),
(361092,36101,0,'Felwood Ore Spawnpoint'),
(361102,36101,0,'Felwood Ore Spawnpoint'),
(361104,36101,0,'Felwood Ore Spawnpoint'),
(361106,36101,0,'Felwood Ore Spawnpoint'),
(361108,36101,0,'Felwood Ore Spawnpoint'),
(361114,36101,0,'Felwood Ore Spawnpoint'),
(361118,36101,0,'Felwood Ore Spawnpoint'),
(361121,36101,0,'Felwood Ore Spawnpoint'),
(361123,36101,0,'Felwood Ore Spawnpoint'),
(361129,36101,0,'Felwood Ore Spawnpoint'),
(361137,36101,0,'Felwood Ore Spawnpoint'),
(645019,2007,0,'Spawn Point 205 - Burning Steppes');
/*!40000 ALTER TABLE `pool_pool` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

