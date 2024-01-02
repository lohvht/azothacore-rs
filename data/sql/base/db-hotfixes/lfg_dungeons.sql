/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `lfg_dungeons`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `lfg_dungeons` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `Description` text DEFAULT NULL,
  `Flags` int(11) NOT NULL DEFAULT 0,
  `MinGear` float NOT NULL DEFAULT 0,
  `MaxLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `TargetLevelMax` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MapID` smallint(6) NOT NULL DEFAULT 0,
  `RandomID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ScenarioID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `FinalEncounterID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `BonusReputationAmount` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MentorItemLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredPlayerConditionId` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `TargetLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `TargetLevelMin` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `DifficultyID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `TypeID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Faction` tinyint(4) NOT NULL DEFAULT 0,
  `ExpansionLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `OrderIndex` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `GroupID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CountTank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CountHealer` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CountDamage` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinCountTank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinCountHealer` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinCountDamage` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Subtype` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MentorCharLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `IconTextureFileID` int(11) NOT NULL DEFAULT 0,
  `RewardsBgTextureFileID` int(11) NOT NULL DEFAULT 0,
  `PopupBgTextureFileID` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `lfg_dungeons` WRITE;
/*!40000 ALTER TABLE `lfg_dungeons` DISABLE KEYS */;
/*!40000 ALTER TABLE `lfg_dungeons` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

