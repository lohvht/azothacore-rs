/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `battlemaster_list`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `battlemaster_list` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `GameType` text DEFAULT NULL,
  `ShortDescription` text DEFAULT NULL,
  `LongDescription` text DEFAULT NULL,
  `IconFileDataID` int(11) NOT NULL DEFAULT 0,
  `MapID1` smallint(6) NOT NULL DEFAULT 0,
  `MapID2` smallint(6) NOT NULL DEFAULT 0,
  `MapID3` smallint(6) NOT NULL DEFAULT 0,
  `MapID4` smallint(6) NOT NULL DEFAULT 0,
  `MapID5` smallint(6) NOT NULL DEFAULT 0,
  `MapID6` smallint(6) NOT NULL DEFAULT 0,
  `MapID7` smallint(6) NOT NULL DEFAULT 0,
  `MapID8` smallint(6) NOT NULL DEFAULT 0,
  `MapID9` smallint(6) NOT NULL DEFAULT 0,
  `MapID10` smallint(6) NOT NULL DEFAULT 0,
  `MapID11` smallint(6) NOT NULL DEFAULT 0,
  `MapID12` smallint(6) NOT NULL DEFAULT 0,
  `MapID13` smallint(6) NOT NULL DEFAULT 0,
  `MapID14` smallint(6) NOT NULL DEFAULT 0,
  `MapID15` smallint(6) NOT NULL DEFAULT 0,
  `MapID16` smallint(6) NOT NULL DEFAULT 0,
  `HolidayWorldState` smallint(6) NOT NULL DEFAULT 0,
  `RequiredPlayerConditionID` smallint(6) NOT NULL DEFAULT 0,
  `InstanceType` tinyint(4) NOT NULL DEFAULT 0,
  `GroupsAllowed` tinyint(4) NOT NULL DEFAULT 0,
  `MaxGroupSize` tinyint(4) NOT NULL DEFAULT 0,
  `MinLevel` tinyint(4) NOT NULL DEFAULT 0,
  `MaxLevel` tinyint(4) NOT NULL DEFAULT 0,
  `RatedPlayers` tinyint(4) NOT NULL DEFAULT 0,
  `MinPlayers` tinyint(4) NOT NULL DEFAULT 0,
  `MaxPlayers` tinyint(4) NOT NULL DEFAULT 0,
  `Flags` tinyint(4) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `battlemaster_list` WRITE;
/*!40000 ALTER TABLE `battlemaster_list` DISABLE KEYS */;
/*!40000 ALTER TABLE `battlemaster_list` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

