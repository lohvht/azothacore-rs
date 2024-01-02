/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `sound_kit`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `sound_kit` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `VolumeFloat` float NOT NULL DEFAULT 0,
  `MinDistance` float NOT NULL DEFAULT 0,
  `DistanceCutoff` float NOT NULL DEFAULT 0,
  `Flags` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SoundEntriesAdvancedID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SoundType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `DialogType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `EAXDef` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VolumeVariationPlus` float NOT NULL DEFAULT 0,
  `VolumeVariationMinus` float NOT NULL DEFAULT 0,
  `PitchVariationPlus` float NOT NULL DEFAULT 0,
  `PitchVariationMinus` float NOT NULL DEFAULT 0,
  `PitchAdjust` float NOT NULL DEFAULT 0,
  `BusOverwriteID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxInstances` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `sound_kit` WRITE;
/*!40000 ALTER TABLE `sound_kit` DISABLE KEYS */;
/*!40000 ALTER TABLE `sound_kit` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

