/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `chr_races`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `chr_races` (
  `ClientPrefix` text DEFAULT NULL,
  `ClientFileString` text DEFAULT NULL,
  `Name` text DEFAULT NULL,
  `NameFemale` text DEFAULT NULL,
  `NameLowercase` text DEFAULT NULL,
  `NameFemaleLowercase` text DEFAULT NULL,
  `Flags` int(11) NOT NULL DEFAULT 0,
  `MaleDisplayId` int(10) unsigned NOT NULL DEFAULT 0,
  `FemaleDisplayId` int(10) unsigned NOT NULL DEFAULT 0,
  `CreateScreenFileDataID` int(11) NOT NULL DEFAULT 0,
  `SelectScreenFileDataID` int(11) NOT NULL DEFAULT 0,
  `MaleCustomizeOffset1` float NOT NULL DEFAULT 0,
  `MaleCustomizeOffset2` float NOT NULL DEFAULT 0,
  `MaleCustomizeOffset3` float NOT NULL DEFAULT 0,
  `FemaleCustomizeOffset1` float NOT NULL DEFAULT 0,
  `FemaleCustomizeOffset2` float NOT NULL DEFAULT 0,
  `FemaleCustomizeOffset3` float NOT NULL DEFAULT 0,
  `LowResScreenFileDataID` int(11) NOT NULL DEFAULT 0,
  `StartingLevel` int(11) NOT NULL DEFAULT 0,
  `UiDisplayOrder` int(11) NOT NULL DEFAULT 0,
  `FactionID` smallint(6) NOT NULL DEFAULT 0,
  `ResSicknessSpellID` smallint(6) NOT NULL DEFAULT 0,
  `SplashSoundID` smallint(6) NOT NULL DEFAULT 0,
  `CinematicSequenceID` smallint(6) NOT NULL DEFAULT 0,
  `BaseLanguage` tinyint(4) NOT NULL DEFAULT 0,
  `CreatureType` tinyint(4) NOT NULL DEFAULT 0,
  `Alliance` tinyint(4) NOT NULL DEFAULT 0,
  `RaceRelated` tinyint(4) NOT NULL DEFAULT 0,
  `UnalteredVisualRaceID` tinyint(4) NOT NULL DEFAULT 0,
  `CharComponentTextureLayoutID` tinyint(4) NOT NULL DEFAULT 0,
  `DefaultClassID` tinyint(4) NOT NULL DEFAULT 0,
  `NeutralRaceID` tinyint(4) NOT NULL DEFAULT 0,
  `DisplayRaceID` tinyint(4) NOT NULL DEFAULT 0,
  `CharComponentTexLayoutHiResID` tinyint(4) NOT NULL DEFAULT 0,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `HighResMaleDisplayId` int(10) unsigned NOT NULL DEFAULT 0,
  `HighResFemaleDisplayId` int(10) unsigned NOT NULL DEFAULT 0,
  `HeritageArmorAchievementID` int(11) NOT NULL DEFAULT 0,
  `MaleSkeletonFileDataID` int(11) NOT NULL DEFAULT 0,
  `FemaleSkeletonFileDataID` int(11) NOT NULL DEFAULT 0,
  `AlteredFormStartVisualKitID1` int(10) unsigned NOT NULL DEFAULT 0,
  `AlteredFormStartVisualKitID2` int(10) unsigned NOT NULL DEFAULT 0,
  `AlteredFormStartVisualKitID3` int(10) unsigned NOT NULL DEFAULT 0,
  `AlteredFormFinishVisualKitID1` int(10) unsigned NOT NULL DEFAULT 0,
  `AlteredFormFinishVisualKitID2` int(10) unsigned NOT NULL DEFAULT 0,
  `AlteredFormFinishVisualKitID3` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `chr_races` WRITE;
/*!40000 ALTER TABLE `chr_races` DISABLE KEYS */;
/*!40000 ALTER TABLE `chr_races` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

