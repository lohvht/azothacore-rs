/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `battleground_template`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `battleground_template` (
  `ID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `MinPlayersPerTeam` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxPlayersPerTeam` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinLvl` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxLvl` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AllianceStartLoc` mediumint(8) unsigned NOT NULL,
  `HordeStartLoc` mediumint(8) unsigned NOT NULL,
  `StartMaxDist` float NOT NULL DEFAULT 0,
  `Weight` tinyint(3) unsigned NOT NULL DEFAULT 1,
  `ScriptName` char(64) NOT NULL DEFAULT '',
  `Comment` char(32) NOT NULL,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `battleground_template` WRITE;
/*!40000 ALTER TABLE `battleground_template` DISABLE KEYS */;
INSERT INTO `battleground_template` VALUES
(1,10,40,45,100,611,610,150,1,'','Alterac Valley'),
(2,5,10,10,100,769,770,75,1,'','Warsong Gulch'),
(3,8,15,10,100,890,889,75,1,'','Arathi Basin'),
(4,0,5,10,100,929,936,0,1,'','Nagrand Arena'),
(5,0,5,10,100,939,940,0,1,'','Blades\'s Edge Arena'),
(6,0,5,10,100,0,0,0,1,'','All Arena'),
(7,8,15,35,100,1103,1104,75,1,'','Eye of The Storm'),
(8,0,5,10,100,1258,1259,0,1,'','Ruins of Lordaeron'),
(9,8,15,65,100,1367,1368,0,1,'','Strand of the Ancients'),
(10,0,5,10,100,1362,1363,0,1,'','Dalaran Sewers'),
(11,0,5,10,100,1364,1365,0,1,'','The Ring of Valor'),
(30,10,40,71,100,1299,1245,200,1,'','Isle of Conquest'),
(32,5,40,45,100,0,0,0,1,'','Random battleground'),
(108,5,10,85,100,1726,1727,0,0,'','Twin Peaks'),
(120,5,10,85,100,1798,1799,0,0,'','The Battle for Gilneas');
/*!40000 ALTER TABLE `battleground_template` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

