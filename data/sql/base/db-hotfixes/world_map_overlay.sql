/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `world_map_overlay`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `world_map_overlay` (
  `TextureName` text DEFAULT NULL,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `TextureWidth` smallint(5) unsigned NOT NULL DEFAULT 0,
  `TextureHeight` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MapAreaID` int(10) unsigned NOT NULL DEFAULT 0,
  `OffsetX` int(11) NOT NULL DEFAULT 0,
  `OffsetY` int(11) NOT NULL DEFAULT 0,
  `HitRectTop` int(11) NOT NULL DEFAULT 0,
  `HitRectLeft` int(11) NOT NULL DEFAULT 0,
  `HitRectBottom` int(11) NOT NULL DEFAULT 0,
  `HitRectRight` int(11) NOT NULL DEFAULT 0,
  `PlayerConditionID` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaID1` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaID2` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaID3` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaID4` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `world_map_overlay` WRITE;
/*!40000 ALTER TABLE `world_map_overlay` DISABLE KEYS */;
/*!40000 ALTER TABLE `world_map_overlay` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

