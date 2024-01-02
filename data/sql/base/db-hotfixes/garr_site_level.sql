/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `garr_site_level`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `garr_site_level` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `TownHallUiPosX` float NOT NULL DEFAULT 0,
  `TownHallUiPosY` float NOT NULL DEFAULT 0,
  `MapID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UiTextureKitID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UpgradeMovieID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UpgradeCost` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UpgradeGoldCost` smallint(5) unsigned NOT NULL DEFAULT 0,
  `GarrLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `GarrSiteID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxBuildingLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `garr_site_level` WRITE;
/*!40000 ALTER TABLE `garr_site_level` DISABLE KEYS */;
/*!40000 ALTER TABLE `garr_site_level` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

