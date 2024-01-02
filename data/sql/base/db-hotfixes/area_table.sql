/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `area_table`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `area_table` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `ZoneName` text DEFAULT NULL,
  `AreaName` text DEFAULT NULL,
  `Flags1` int(11) NOT NULL DEFAULT 0,
  `Flags2` int(11) NOT NULL DEFAULT 0,
  `AmbientMultiplier` float NOT NULL DEFAULT 0,
  `ContinentID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ParentAreaID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AreaBit` smallint(6) NOT NULL DEFAULT 0,
  `AmbienceID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ZoneMusic` smallint(5) unsigned NOT NULL DEFAULT 0,
  `IntroSound` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LiquidTypeID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LiquidTypeID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LiquidTypeID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LiquidTypeID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UwZoneMusic` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UwAmbience` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PvpCombatWorldStateID` smallint(6) NOT NULL DEFAULT 0,
  `SoundProviderPref` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `SoundProviderPrefUnderwater` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ExplorationLevel` tinyint(4) NOT NULL DEFAULT 0,
  `FactionGroupMask` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MountFlags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `WildBattlePetLevelMin` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `WildBattlePetLevelMax` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `WindSettingsID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `UwIntroSound` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `area_table` WRITE;
/*!40000 ALTER TABLE `area_table` DISABLE KEYS */;
/*!40000 ALTER TABLE `area_table` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

