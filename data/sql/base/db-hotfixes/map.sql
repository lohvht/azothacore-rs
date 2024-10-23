/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `map`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `map` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Directory` text DEFAULT NULL,
  `MapName` text DEFAULT NULL,
  `MapDescription0` text DEFAULT NULL,
  `MapDescription1` text DEFAULT NULL,
  `PvpShortDescription` text DEFAULT NULL,
  `PvpLongDescription` text DEFAULT NULL,
  `Flags1` int(11) NOT NULL DEFAULT 0,
  `Flags2` int(11) NOT NULL DEFAULT 0,
  `MinimapIconScale` float NOT NULL DEFAULT 0,
  `Corpse1` float NOT NULL DEFAULT 0,
  `Corpse2` float NOT NULL DEFAULT 0,
  `AreaTableID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LoadingScreenID` smallint(6) NOT NULL DEFAULT 0,
  `CorpseMapID` smallint(6) NOT NULL DEFAULT 0,
  `TimeOfDayOverride` smallint(6) NOT NULL DEFAULT 0,
  `ParentMapID` smallint(6) NOT NULL DEFAULT 0,
  `CosmeticParentMapID` smallint(6) NOT NULL DEFAULT 0,
  `WindSettingsID` smallint(6) NOT NULL DEFAULT 0,
  `InstanceType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MapType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ExpansionID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxPlayers` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `TimeOffset` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `map` WRITE;
/*!40000 ALTER TABLE `map` DISABLE KEYS */;
/*!40000 ALTER TABLE `map` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

