/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `creature_display_info`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `creature_display_info` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `CreatureModelScale` float NOT NULL DEFAULT 0,
  `ModelID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `NPCSoundID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SizeClass` tinyint(4) NOT NULL DEFAULT 0,
  `Flags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Gender` tinyint(4) NOT NULL DEFAULT 0,
  `ExtendedDisplayInfoID` int(11) NOT NULL DEFAULT 0,
  `PortraitTextureFileDataID` int(11) NOT NULL DEFAULT 0,
  `CreatureModelAlpha` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `SoundID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PlayerOverrideScale` float NOT NULL DEFAULT 0,
  `PortraitCreatureDisplayInfoID` int(11) NOT NULL DEFAULT 0,
  `BloodID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ParticleColorID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CreatureGeosetData` int(10) unsigned NOT NULL DEFAULT 0,
  `ObjectEffectPackageID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AnimReplacementSetID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `UnarmedWeaponType` tinyint(4) NOT NULL DEFAULT 0,
  `StateSpellVisualKitID` int(11) NOT NULL DEFAULT 0,
  `PetInstanceScale` float NOT NULL DEFAULT 0,
  `MountPoofSpellVisualKitID` int(11) NOT NULL DEFAULT 0,
  `TextureVariationFileDataID1` int(11) NOT NULL DEFAULT 0,
  `TextureVariationFileDataID2` int(11) NOT NULL DEFAULT 0,
  `TextureVariationFileDataID3` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `creature_display_info` WRITE;
/*!40000 ALTER TABLE `creature_display_info` DISABLE KEYS */;
/*!40000 ALTER TABLE `creature_display_info` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

