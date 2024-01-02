/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `world_map_area`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `world_map_area` (
  `AreaName` text DEFAULT NULL,
  `LocLeft` float NOT NULL DEFAULT 0,
  `LocRight` float NOT NULL DEFAULT 0,
  `LocTop` float NOT NULL DEFAULT 0,
  `LocBottom` float NOT NULL DEFAULT 0,
  `Flags` int(10) unsigned NOT NULL DEFAULT 0,
  `MapID` smallint(6) NOT NULL DEFAULT 0,
  `AreaID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `DisplayMapID` smallint(6) NOT NULL DEFAULT 0,
  `DefaultDungeonFloor` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ParentWorldMapID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LevelRangeMin` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LevelRangeMax` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `BountySetID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `BountyDisplayLocation` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `VisibilityPlayerConditionID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `world_map_area` WRITE;
/*!40000 ALTER TABLE `world_map_area` DISABLE KEYS */;
/*!40000 ALTER TABLE `world_map_area` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

